use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn};
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{Anchor, CrossAlign, Dismiss, LayerEvent, LayerSize, LayerSpec, Side};
pub use crate::picker::{AsItem, Item, ItemRow};
use crate::response::{Response, StateFlags};
use crate::text::TextBuffer;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_collections::collection::{CollectionCore, Reconcile};
use termrock_runtime::empty::EmptyState;
use termrock_runtime::layer::{LayerOpening, UpdateToken};
use termrock_runtime::rowui::RowFn;
use termrock_runtime::scroll_region::ScrollRegion;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompletionAction {
    Moved,
    Accepted(ItemKey),
    Dismissed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompletionCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Accept,
    Dismiss,
}

/// Baseline completion menus sit four cells left of the replaced token.
const COMPLETION_TEXT_INSET: u16 = 4;

fn popup_anchor(anchor: Rect, replace_len: usize) -> Rect {
    let replace = u16::try_from(replace_len).unwrap_or(u16::MAX);
    Rect {
        x: anchor
            .x
            .saturating_sub(replace)
            .saturating_sub(COMPLETION_TEXT_INSET),
        ..anchor
    }
}

fn paint_completion_row(ui: &mut Ui<'_>, row: Rect, item: crate::picker::Item<'_>, chosen: bool) {
    use crate::theme::{FgStep, Role};
    use ratatui_core::style::Modifier;
    let card = Role::Surface(crate::theme::Surface::Elevated);
    let highlight = Role::HighlightBg;
    let row_bg = if chosen { highlight } else { card };
    let mut fill = StylePatch::new()
        .set_bg(row_bg)
        .set_fg(Role::Fg(FgStep::Primary));
    if chosen {
        fill = fill.add(Modifier::BOLD);
    }
    ui.fill(row, ui.paint_patch(&fill));
    if row.width > 0 {
        let mut pad = StylePatch::new().set_fg(row_bg).set_bg(row_bg);
        if chosen {
            pad = pad.add(Modifier::BOLD);
        }
        ui.paint_str(Rect { width: 1, ..row }, " ", ui.paint_patch(&pad));
    }
    let glyph_fg = if chosen {
        Role::Fg(FgStep::Primary)
    } else {
        Role::Fg(FgStep::Muted)
    };
    if row.width > 1 {
        ui.paint_str(
            Rect {
                x: row.x.saturating_add(1),
                width: 1,
                ..row
            },
            if item.glyph.is_empty() {
                " "
            } else {
                item.glyph
            },
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(glyph_fg)
                    .set_bg(row_bg)
                    .remove(Modifier::BOLD),
            ),
        );
    }
    let detail_w = crate::text::width(item.detail);
    let show_detail = !item.detail.is_empty()
        && usize::from(row.width.saturating_sub(3))
            > crate::text::width(item.label) as usize + detail_w as usize + 2;
    let label_x = row.x.saturating_add(3);
    let mut x = label_x;
    for (index, ch) in item.label.char_indices() {
        if x >= row.right() {
            break;
        }
        let mut patch = StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(row_bg);
        patch = if item.matched.contains(&index) || chosen {
            patch.add(Modifier::BOLD)
        } else {
            patch.remove(Modifier::BOLD)
        };
        let mut bytes = [0; 4];
        let text = ch.encode_utf8(&mut bytes);
        ui.paint_str(Rect { x, width: 1, ..row }, text, ui.paint_patch(&patch));
        x = x.saturating_add(crate::text::width(text));
    }
    if show_detail {
        let dx = row.right().saturating_sub(detail_w.saturating_add(1));
        let detail_fg = if chosen {
            Role::Fg(FgStep::Primary)
        } else {
            Role::Fg(FgStep::Muted)
        };
        ui.paint_str(
            Rect {
                x: dx,
                width: detail_w,
                ..row
            },
            item.detail,
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(detail_fg)
                    .set_bg(row_bg)
                    .remove(Modifier::BOLD),
            ),
        );
    }
}

/// Baseline menus size to the longest label plus the longest detail.
/// Those two strings may sit on different rows.
fn popup_width<T: AsItem>(cx: &Cx<'_>, items: &[T]) -> u16 {
    let mut label_w = 0u16;
    let mut detail_w = 0u16;
    for item in items {
        let item = item.as_item();
        label_w = label_w.max(crate::text::width(item.label));
        detail_w = detail_w.max(crate::text::width(item.detail));
    }
    label_w
        .saturating_add(detail_w)
        .saturating_add(8)
        .clamp(24, 48)
        .min(cx.design().size.popup_max_width)
}

const fn binding(
    action: ActionKey,
    chord: Chord,
    cmd: CompletionCmd,
    label: &'static str,
    visible: bool,
) -> Binding<CompletionCmd> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 70 } else { 10 },
        visible,
    }
}

const MOVE_DOWN: ActionKey = ActionKey::custom("completion.down");

const BINDINGS: &[Binding<CompletionCmd>] = &[
    binding(
        ActionKey::custom("completion.up"),
        Chord::key(KeyCode::Up),
        CompletionCmd::Up,
        "Up",
        false,
    ),
    binding(
        MOVE_DOWN,
        Chord::key(KeyCode::Down),
        CompletionCmd::Down,
        "Down",
        false,
    ),
    binding(
        ActionKey::custom("completion.ctrl-up"),
        Chord::with(KeyCode::Char('p'), KeyModifiers::CONTROL),
        CompletionCmd::Up,
        "Up",
        false,
    ),
    binding(
        ActionKey::custom("completion.ctrl-down"),
        Chord::with(KeyCode::Char('n'), KeyModifiers::CONTROL),
        CompletionCmd::Down,
        "Down",
        false,
    ),
    binding(
        ActionKey::custom("completion.page-up"),
        Chord::key(KeyCode::PageUp),
        CompletionCmd::PageUp,
        "Page up",
        false,
    ),
    binding(
        ActionKey::custom("completion.page-down"),
        Chord::key(KeyCode::PageDown),
        CompletionCmd::PageDown,
        "Page down",
        false,
    ),
    binding(
        ActionKey::custom("completion.accept-tab"),
        Chord::key(KeyCode::Tab),
        CompletionCmd::Accept,
        "Accept",
        true,
    ),
    binding(
        ActionKey::custom("completion.accept"),
        Chord::key(KeyCode::Enter),
        CompletionCmd::Accept,
        "Accept",
        true,
    ),
    binding(
        ActionKey::custom("completion.dismiss"),
        Chord::key(KeyCode::Esc),
        CompletionCmd::Dismiss,
        "Dismiss",
        false,
    ),
];

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CompletionState {
    core: CollectionCore,
    open: bool,
    owner: Option<Id>,
    anchor: Rect,
    replace_len: usize,
    opening: Option<LayerOpening>,
    programmatic_close: Option<(LayerOpening, UpdateToken)>,
}

impl CompletionState {
    pub const fn is_open(&self) -> bool {
        self.open
    }
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }
    pub const fn replace_len(&self) -> usize {
        self.replace_len
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CompletionController {
    editor_id: Id,
    popup_id: Id,
}

impl CompletionController {
    pub const fn new(editor_id: Id, popup_id: Id) -> Self {
        Self {
            editor_id,
            popup_id,
        }
    }

    pub fn request<T: AsItem>(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        anchor: Rect,
        replace_len: usize,
        items: &[T],
    ) {
        state.open = !items.is_empty();
        state.owner = state.open.then_some(self.editor_id);
        let anchor = popup_anchor(anchor, replace_len);
        state.anchor = anchor;
        state.replace_len = replace_len;
        state.core.invalidate();
        if state.open {
            let rows = items.len().clamp(1, 8) as u16;
            let size = LayerSize::Fixed(popup_width(cx, items), rows.saturating_add(2));
            let anchor = Anchor::Rect {
                rect: anchor,
                side: Side::Below,
                align: CrossAlign::Start,
            };
            if cx.is_open(self.popup_id) {
                cx.reanchor_layer(self.popup_id, anchor);
                cx.resize_layer(self.popup_id, size);
            } else {
                // Completion is non-modal and leaves focus on its editor
                // owner. Restoring the opener on close would undo a real
                // focus move made while the popup was open.
                let _ = cx.open_layer_with_token(
                    self.popup_id,
                    LayerSpec::popover(self.popup_id, anchor)
                        .restore_focus(false)
                        .dismiss(Dismiss::ESC_AND_OUTSIDE)
                        .size(size),
                );
            }
            state.opening = cx.layer_opening(self.popup_id).or(state.opening);
        } else {
            if let Some(opening) = cx.layer_opening(self.popup_id) {
                state.opening = Some(opening);
                state.programmatic_close = Some((opening, cx.update_token()));
            }
            cx.close_layer(self.popup_id, None);
        }
    }

    pub fn dismiss(&self, cx: &mut Cx<'_>, state: &mut CompletionState) {
        state.open = false;
        state.owner = None;
        if let Some(opening) = cx.layer_opening(self.popup_id) {
            state.opening = Some(opening);
            state.programmatic_close = Some((opening, cx.update_token()));
        }
        cx.close_layer(self.popup_id, None);
    }

    pub fn dismiss_on_editor_motion(&self, cx: &mut Cx<'_>, state: &mut CompletionState) -> bool {
        if !state.open {
            return false;
        }
        self.dismiss(cx, state);
        true
    }

    pub fn accept(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        buffer: &mut TextBuffer,
        item: Item<'_>,
    ) -> bool {
        self.accept_with(cx, state, item, |replace_len, insertion| {
            Self::splice(buffer, replace_len, insertion)
        })
    }

    /// Accept an item through a caller-owned editor splice operation.
    ///
    /// The completion controller still owns dismissal; the adapter owns how the
    /// editor applies the byte range and insertion. This keeps editor state and
    /// selection lifecycle in the editor crate without adding a dependency here.
    pub fn accept_with(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        item: Item<'_>,
        splice: impl FnOnce(usize, &str) -> bool,
    ) -> bool {
        let inserted = splice(state.replace_len, item.insertion());
        self.dismiss(cx, state);
        inserted
    }

    fn splice(buffer: &mut TextBuffer, replace_len: usize, insertion: &str) -> bool {
        let end = buffer.cursor_offset();
        let start = end.saturating_sub(replace_len);
        buffer.select_range(start, end);
        buffer.insert_str(insertion)
    }
}

pub struct Completion<'a, T, R = ItemRow> {
    id: Id,
    #[allow(dead_code)]
    row: R,
    empty: Option<EmptyState<'a>>,
    max_rows: u16,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
    _item: PhantomData<fn(&T)>,
}

impl<T, R> core::fmt::Debug for Completion<'_, T, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Completion")
            .field("id", &self.id)
            .field("max_rows", &self.max_rows)
            .finish_non_exhaustive()
    }
}

impl<T> Completion<'_, T, ItemRow> {
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            row: ItemRow,
            empty: None,
            max_rows: 8,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _item: PhantomData,
        }
    }
}

impl<'a, T, R> Completion<'a, T, R> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::GUTTER,
        Part::ICON,
        Part::ROW,
        Part::LABEL,
        Part::META,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];
    pub const fn id(&self) -> Id {
        self.id
    }
    pub fn row<R2: RowFn<T>>(self, row: R2) -> Completion<'a, T, R2> {
        Completion {
            id: self.id,
            row,
            empty: self.empty,
            max_rows: self.max_rows,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }
    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    #[must_use]
    pub const fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = if rows == 0 { 1 } else { rows };
        self
    }
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.global(patch);
        self
    }
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self.ov = self.ov.part(parts);
        self
    }
    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }
    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut bar = ScrollRegion::new(self.id).patch_part(self.parts);
        if let Some(patch) = self.patch {
            bar = bar.patch(patch);
        }
        if let Some(slot) = self.ov.slot_for(Part::TRACK) {
            bar = bar.slot(Part::TRACK, slot);
        } else if let Some(slot) = self.ov.slot_for(Part::THUMB) {
            bar = bar.slot(Part::THUMB, slot);
        }
        bar
    }
}

impl<T: AsItem, R: RowFn<T>> Completion<'_, T, R> {
    fn key_at(items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |item| item.as_item().key)
    }
    fn move_to(
        state: &mut CompletionState,
        items: &[T],
        target: usize,
        acc: &mut Acc<CompletionAction>,
    ) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let i = target.min(items.len().saturating_sub(1));
        let key = Self::key_at(items, i);
        if state.core.cursor() == Some(key) {
            acc.consumed();
            return;
        }
        state.core.set_cursor(i, key);
        acc.action(CompletionAction::Moved);
    }

    fn handle_editor_intent(
        &self,
        controller: CompletionController,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        items: &[T],
        intent: Intent<'_>,
        acc: &mut Acc<CompletionAction>,
    ) {
        match intent {
            Intent::Binding(action) => match Binding::command(BINDINGS, action) {
                Some(cmd) => {
                    let cur = state.core.cursor_index();
                    let page = state.core.scroll().viewport_len().max(1);
                    match cmd {
                        CompletionCmd::Up => {
                            Self::move_to(state, items, cur.saturating_sub(1), acc);
                        }
                        CompletionCmd::Down => {
                            Self::move_to(state, items, cur.saturating_add(1), acc);
                        }
                        CompletionCmd::PageUp => {
                            Self::move_to(state, items, cur.saturating_sub(page), acc);
                        }
                        CompletionCmd::PageDown => {
                            Self::move_to(state, items, cur.saturating_add(page), acc);
                        }
                        CompletionCmd::Accept => match items.get(cur).map(AsItem::as_item) {
                            Some(item) if !item.disabled => {
                                acc.action(CompletionAction::Accepted(item.key));
                            }
                            _ => acc.consumed(),
                        },
                        CompletionCmd::Dismiss => {
                            state.open = false;
                            state.owner = None;
                            cx.close_layer(self.id, None);
                            acc.action(CompletionAction::Dismissed);
                        }
                    }
                }
                None => {
                    if controller.dismiss_on_editor_motion(cx, state) {
                        acc.action(CompletionAction::Dismissed);
                    }
                }
            },
            Intent::FocusOut { .. } | Intent::Cancel
                if controller.dismiss_on_editor_motion(cx, state) =>
            {
                acc.action(CompletionAction::Dismissed);
            }

            _ => {}
        }
    }

    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        let width = popup_width(cx, items);
        let rows = items
            .len()
            .min(usize::from(self.max_rows))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        LayerSize::Fixed(width, rows.saturating_add(2))
    }

    pub fn update_for(
        &self,
        owner: Id,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        items: &[T],
    ) -> Response<CompletionAction> {
        let mut events = Acc::<CompletionAction>::new();
        while let Some(receipt) = cx.take_layer_intent_receipt(self.id) {
            let opening = receipt.opening();
            let expected_close = state.programmatic_close.is_some_and(|(expected, update)| {
                expected == opening && receipt.update() == Some(update)
            });
            if expected_close {
                state.programmatic_close = None;
                if state.opening == Some(opening) && !state.open {
                    state.opening = None;
                }
                continue;
            }
            if state.opening != Some(opening)
                || cx
                    .layer_opening(self.id)
                    .is_some_and(|live| live != opening)
            {
                continue;
            }
            match receipt.event() {
                LayerEvent::Dismissed(_) => {
                    let was_open = state.open;
                    state.open = false;
                    state.owner = None;
                    state.opening = None;
                    if was_open {
                        events.action(CompletionAction::Dismissed);
                    }
                }
                LayerEvent::Opened | LayerEvent::Closed(_) => {}
            }
        }
        if !state.open {
            return events.finish(self.id);
        }
        state.owner = Some(owner);
        let _ = state
            .core
            .reconcile(items.len(), |i| Self::key_at(items, i));
        if state.core.cursor().is_none() && !items.is_empty() {
            state.core.set_cursor(0, Self::key_at(items, 0));
        }
        cx.resize_layer(self.id, self.measured_size(cx, items));
        cx.reanchor_layer(
            self.id,
            Anchor::Rect {
                rect: state.anchor,
                side: Side::Below,
                align: CrossAlign::Start,
            },
        );
        let bar = self
            .scrollbar()
            .update(cx, state.core.scroll_mut(), items.len());
        let mut acc = Acc::new();
        acc.fold(&bar);
        let controller = CompletionController::new(owner, self.id);
        for intent in cx.intents(owner) {
            self.handle_editor_intent(controller, cx, state, items, intent, &mut acc);
        }
        for intent in cx.intents(self.id) {
            match intent {
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(key),
                        },
                    ..
                } => match items.iter().position(|item| item.as_item().key == key) {
                    Some(i) if !items.get(i).is_some_and(|item| item.as_item().disabled) => {
                        state.core.set_cursor(i, key);
                        acc.action(CompletionAction::Accepted(key));
                    }
                    _ => {
                        acc.consumed();
                    }
                },
                Intent::Pointer { .. } => {
                    acc.consumed();
                }
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CompletionState, items: &[T]) -> Rect {
        ui.with_surface(crate::theme::Surface::Elevated, |ui| {
            if state.open
                && let Some(owner) = state.owner
            {
                let live = ui.state(owner);
                ui.publish_bindings(owner, live, self.bindings(BindingState { flags: live }));
            }
            let base = self.ov.style(
                ui,
                self.id,
                Family::COMPLETION,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::empty(),
            );
            ui.fill(area, base.style);
            let border = self.ov.style(
                ui,
                self.id,
                Family::COMPLETION,
                Variant::DEFAULT,
                Part::BORDER,
                StateFlags::empty(),
            );
            let border_style = border
                .style
                .patch(ui.paint_patch(&StylePatch::new().set_fg(crate::theme::Role::BorderStrong)));
            let inner = ui.frame(area, border_style);
            if items.is_empty() {
                self.empty
                    .unwrap_or(EmptyState::Empty {
                        title: "No completions",
                        hint: None,
                    })
                    .draw(ui, inner, 0);
                return area;
            }
            let content = self
                .scrollbar()
                .draw(ui, inner, state.core.scroll(), items.len());
            let view = ScrollRegion::view(state.core.scroll(), content, items.len());
            for (row_i, i) in view.visible_range().enumerate() {
                let Some(item) = items.get(i) else {
                    break;
                };
                let semantic = item.as_item();
                let row = Rect {
                    x: content.x,
                    y: content
                        .y
                        .saturating_add(row_i.min(usize::from(u16::MAX)) as u16),
                    width: content.width,
                    height: 1,
                };
                let chosen = state.core.cursor() == Some(semantic.key);
                paint_completion_row(ui, row, semantic, chosen);
                ui.register_part(self.id, PartRef::item(Part::ROW, semantic.key), row);
            }
            // Baseline option lists fade the last visible row when more items sit below.
            let mut faded = *state.core.scroll();
            faded.apply_layout(usize::from(content.height), items.len());
            ui.scroll_edges(content, &faded);
            area
        })
    }
}

impl<T, R> Bindings for Completion<'_, T, R> {
    type Cmd = CompletionCmd;
    fn bindings(&self, _state: BindingState) -> &'static [Binding<Self::Cmd>] {
        BINDINGS
    }
}

#[cfg(test)]
mod width_tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use termrock_runtime::event::Input;
    use termrock_runtime::response::Response;
    use termrock_runtime::runtime::stub;
    use termrock_runtime::runtime::{App, Runtime};
    use termrock_theme::Theme;

    use super::Completion;
    use crate::id::{Id, ItemKey};
    use crate::layer::LayerSize;
    use crate::picker::Item;
    use crate::ui::{Cx, Ui};

    const SPLIT: [Item<'static>; 2] = [
        Item::new(ItemKey::num(1), "shipped_at").detail("timestamptz · null"),
        Item::new(ItemKey::num(2), "shipping_address").detail("json"),
    ];

    #[test]
    fn width_adds_the_longest_label_to_the_longest_detail() {
        assert_eq!(measure(&SPLIT), LayerSize::Fixed(42, 4));
    }

    fn measure(items: &'static [Item<'static>]) -> LayerSize {
        struct MeasureApp {
            items: &'static [Item<'static>],
            measured: Option<LayerSize>,
        }
        impl App for MeasureApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                self.measured = Some(
                    Completion::<Item<'static>>::new(Id::root("completion.width"))
                        .measured_size(cx, self.items),
                );
                Response::ignored()
            }

            fn draw(&self, _ui: &mut Ui<'_>) {}
        }
        let area = Rect::new(0, 0, 100, 30);
        let mut runtime = Runtime::new(
            MeasureApp {
                items,
                measured: None,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = stub::deliver(&mut runtime, Input::Tick);
        runtime.app().measured.unwrap_or(LayerSize::Fixed(0, 0))
    }
}
