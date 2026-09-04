//! Multi-column key-reference overlay (`COMPONENT_ARCHITECTURE.md` §14.2
//! J5, §18.3 #18, Appendix A 4F).

use core::fmt;

use ratatui_core::layout::Rect;

use super::keyhint::ChordText;
use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn, first_row, shift};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::Intent;
use crate::keymap::{Binding, BindingState, Bindings, HintLayer};
use crate::layer::{DismissReason, LayerEvent, LayerSize, LayerSpec};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::theme::{Family, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// One titled group, borrowing the same derived binding metadata a
/// [`crate::components::HintBar`] consumes.
#[derive(Clone, Copy, Debug)]
pub struct HelpSection<'a> {
    title: &'a str,
    layer: &'a HintLayer,
}

impl<'a> HelpSection<'a> {
    /// A section over a [`HintLayer::from_bindings`] result.
    pub const fn new(title: &'a str, layer: &'a HintLayer) -> Self {
        HelpSection { title, layer }
    }

    /// Heading.
    pub const fn title(&self) -> &'a str {
        self.title
    }

    /// Derived key bindings.
    pub const fn layer(&self) -> &'a HintLayer {
        self.layer
    }

    const fn rows(&self) -> usize {
        self.layer.hints.len().saturating_add(2)
    }
}

/// Durable help viewport state.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct HelpOverlayState {
    scroll: ScrollState,
}

impl HelpOverlayState {
    /// Scroll position.
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }
}

/// Help-overlay output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HelpAction {
    /// Overlay closed by its close key or the layer policy.
    Closed(DismissReason),
}

/// Help navigation commands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HelpCmd {
    /// One row up.
    Prev,
    /// One row down.
    Next,
    /// One page up.
    PageUp,
    /// One page down.
    PageDown,
    /// Close the overlay.
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

/// Scrollable key reference. Sections are assigned round-robin across one
/// to three 36-cell columns, preserving each section as one block.
///
/// ## Construction
/// Build [`HintLayer`]s from component binding tables, wrap them in
/// [`HelpSection`]s, then call `HelpOverlay::new(id, scope, sections)`.
/// Open [`HelpOverlay::layer`] and draw inside `ui.layer(id, ...)`.
///
/// ## Ownership
/// Caller owns derived binding layers, sections and [`HelpOverlayState`].
/// Runtime owns modal lifecycle, focus trap, backdrop and pointer barrier.
///
/// ## Configuration
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::HELP`, `Variant::DEFAULT`.
///
/// ## States
/// Modal border wears `FOCUSED`; runtime hover/press do not change reference
/// rows because they are decorative.
///
/// ## Actions
/// [`HelpAction::Closed`] reports close-key and layer dismissal reasons.
///
/// ## Focus
/// One focusable modal stop, trapped by the runtime's modal scope.
///
/// ## Keyboard
/// Up/down (`k`/`j`), PageUp/PageDown; `?`, `q`, Enter close. Esc belongs to
/// layer dismissal policy.
///
/// ## Mouse
/// Wheel and scrollbar intents route through [`ScrollRegion`]. Reference
/// rows are decorative.
///
/// ## Layout
/// Modal requests wide-dialog width. Resolved body chooses 1–3 columns at
/// 36 cells and assigns whole sections round-robin. All columns share one
/// vertical scroll offset.
///
/// ## Parts
/// `CONTAINER`, `BORDER`, `TITLE`, `DETAIL` (scope), `HEADER`, `KEY`,
/// `ACTION`, `TRACK`, `THUMB`.
///
/// ## Overrides
/// `.patch` and `.patch_part` reach every part. Slots reach text, border and
/// scrollbar chrome; `CONTAINER` remains the owned surface fill.
///
/// ## Identity
/// One overlay `Id`; reference rows are facts, not actionable keyed items.
///
/// ## Testing
/// `HelpOverlayCase`; `render::components::help_overlay::*`.
///
/// ## Invariants
/// Help and hint bar consume the same derived [`HintLayer`] metadata.
/// Sections never split between columns. Sizing is asserted from update.
pub struct HelpOverlay<'a> {
    id: Id,
    scope: &'a str,
    sections: &'a [HelpSection<'a>],
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: Overrides<'a>,
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
    /// Natural column width and maximum column count from the accepted
    /// migrated control.
    pub const COLUMN_WIDTH: u16 = 36;
    /// Maximum parallel columns.
    pub const MAX_COLUMNS: usize = 3;

    /// Styled parts.
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

    /// A scoped key reference over derived binding layers.
    pub const fn new(id: Id, scope: &'a str, sections: &'a [HelpSection<'a>]) -> Self {
        HelpOverlay {
            id,
            scope,
            sections,
            patch: None,
            parts: &[],
            ov: Overrides::new(),
        }
    }

    /// Instance patch.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.patch(patch);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self.ov = self.ov.patch_part(patches);
        self
    }

    /// Replace one paintable part.
    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    /// Fixed request supplied to the one layer resolver.
    pub fn measured_size(&self, cx: &Cx<'_>) -> LayerSize {
        let d = cx.design();
        let cols = columns_for(d.size.dialog_width_wide.saturating_sub(2));
        let content = self.column_rows(cols).min(usize::from(u16::MAX)) as u16;
        let max_body = d.size.popup_max_rows.saturating_mul(2).max(8);
        LayerSize::Fixed(
            d.size.dialog_width_wide,
            content.min(max_body).saturating_add(4),
        )
    }

    /// Modal layer spec; the runtime owns centring, backdrop, trap and Esc.
    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        LayerSpec::modal(self.id).size(self.measured_size(cx))
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

    /// Re-assert size, update scroll, then drain navigation/lifecycle input.
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

    /// Paint resolved layer content.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &HelpOverlayState) -> Rect {
        if area.is_empty() {
            return area;
        }
        ui.with_surface(Surface::Elevated, |ui| {
            let live = Overrides::flags(ui.state(self.id), StateFlags::empty());
            let container = self.ov.style(
                ui,
                self.id,
                Family::HELP,
                Variant::DEFAULT,
                Part::CONTAINER,
                live,
            );
            ui.fill(area, container.style);
            let border = self.ov.style(
                ui,
                self.id,
                Family::HELP,
                Variant::DEFAULT,
                Part::BORDER,
                live,
            );
            let inner = ui.frame(area, border.style);
            if let Some(slot) = self.ov.slot_for(Part::BORDER) {
                slot(ui, area);
            }
            ui.register_control(self.id, area, Focusability::Focusable);
            ui.publish_bindings(self.id, live, BINDINGS);
            ui.register_decor(self.id, PartRef::of(Part::BORDER), area);
            if inner.is_empty() {
                return;
            }
            let title_row = first_row(inner);
            self.paint_text(ui, Part::TITLE, title_row, "Keyboard shortcuts", live);
            let scope_row = first_row(Rect {
                y: inner.y.saturating_add(1),
                ..inner
            });
            self.paint_text(ui, Part::DETAIL, scope_row, self.scope, live);
            let body = Rect {
                y: inner.y.saturating_add(2),
                height: inner.height.saturating_sub(2),
                ..inner
            };
            let scrollbar = self.scrollbar();
            let columns = columns_for(body.width);
            let content_len = self.column_rows(columns);
            let content = scrollbar.draw(ui, body, &st.scroll, content_len);
            let view = ScrollRegion::view(&st.scroll, content, content_len);
            self.draw_columns(ui, content, &view, columns, live);
            ui.register_decor(self.id, PartRef::of(Part::TITLE), title_row);
            ui.register_decor(self.id, PartRef::of(Part::DETAIL), scope_row);
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

    fn paint_text(&self, ui: &mut Ui<'_>, part: Part, area: Rect, text: &str, flags: StateFlags) {
        let style = self
            .ov
            .style(ui, self.id, Family::HELP, Variant::DEFAULT, part, flags);
        if let Some(slot) = self.ov.slot_for(part) {
            slot(ui, area);
        } else {
            ui.paint_str(area, text, style.style);
        }
    }

    fn draw_columns(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        view: &ScrollState,
        columns: usize,
        flags: StateFlags,
    ) {
        let column_width = area.width.checked_div(columns as u16).unwrap_or(0);
        for column in 0..columns {
            let column_area = Rect {
                x: area
                    .x
                    .saturating_add((column as u16).saturating_mul(column_width)),
                width: if column.saturating_add(1) == columns {
                    area.right().saturating_sub(
                        area.x
                            .saturating_add((column as u16).saturating_mul(column_width)),
                    )
                } else {
                    column_width
                },
                ..area
            };
            self.draw_column(ui, column_area, view, columns, column, flags);
        }
    }

    fn draw_column(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        view: &ScrollState,
        columns: usize,
        column: usize,
        flags: StateFlags,
    ) {
        let start = view.offset();
        let end = start.saturating_add(view.viewport_len());
        let mut virtual_row = 0usize;
        for (index, section) in self.sections.iter().enumerate() {
            if index.checked_rem(columns) != Some(column) {
                continue;
            }
            self.draw_virtual_row(
                ui,
                area,
                virtual_row,
                start,
                end,
                Part::HEADER,
                section.title,
                flags,
            );
            virtual_row = virtual_row.saturating_add(1);
            for hint in &section.layer.hints {
                if start <= virtual_row && virtual_row < end {
                    let y = area
                        .y
                        .saturating_add(virtual_row.saturating_sub(start) as u16);
                    let row = first_row(Rect { y, ..area });
                    let key = Rect {
                        width: row.width.min(12),
                        ..row
                    };
                    let action = shift(row, 13);
                    let chord = ChordText::of(hint.chord);
                    self.paint_text(ui, Part::KEY, key, chord.as_str(), flags);
                    self.paint_text(ui, Part::ACTION, action, hint.label, flags);
                    ui.register_decor(self.id, PartRef::of(Part::KEY), key);
                    ui.register_decor(self.id, PartRef::of(Part::ACTION), action);
                }
                virtual_row = virtual_row.saturating_add(1);
            }
            virtual_row = virtual_row.saturating_add(1);
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "virtual row projection needs source and viewport coordinates"
    )]
    fn draw_virtual_row(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        virtual_row: usize,
        start: usize,
        end: usize,
        part: Part,
        text: &str,
        flags: StateFlags,
    ) {
        if !(start <= virtual_row && virtual_row < end) {
            return;
        }
        let row = first_row(Rect {
            y: area
                .y
                .saturating_add(virtual_row.saturating_sub(start) as u16),
            ..area
        });
        self.paint_text(ui, part, row, text, flags);
        ui.register_decor(self.id, PartRef::of(part), row);
    }

    /// Preferred modal size.
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
