//! Non-modal completion popover and editor coordination.

use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::picker::{AsItem, Item, ItemRow};
use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn};
use crate::action::ActionKey;
use crate::collection::{CollectionCore, EmptyState, Reconcile, RowFn, RowUi};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{Anchor, CrossAlign, Dismiss, LayerEvent, LayerSize, LayerSpec, Side};
use crate::response::{Response, StateFlags};
use crate::text::TextBuffer;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// Completion events.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompletionAction {
    /// Cursor moved.
    Moved,
    /// A semantic item was accepted.
    Accepted(ItemKey),
    /// The popover was dismissed.
    Dismissed,
}

/// Const completion commands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompletionCmd {
    /// Previous suggestion.
    Up,
    /// Next suggestion.
    Down,
    /// Previous page.
    PageUp,
    /// Next page.
    PageDown,
    /// Accept the current suggestion.
    Accept,
    /// Dismiss suggestions.
    Dismiss,
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

/// Durable cursor, scrolling, anchoring and replacement state.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CompletionState {
    core: CollectionCore,
    open: bool,
    owner: Option<Id>,
    anchor: Rect,
    replace_len: usize,
}

impl CompletionState {
    /// Whether suggestions are open.
    pub const fn is_open(&self) -> bool {
        self.open
    }
    /// Current key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }
    /// Bytes before the editor cursor replaced on acceptance.
    pub const fn replace_len(&self) -> usize {
        self.replace_len
    }
}

/// Controller for the editor-to-popover lifecycle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CompletionController {
    editor_id: Id,
    popup_id: Id,
}

impl CompletionController {
    /// Bind one editor owner to its completion popover.
    pub const fn new(editor_id: Id, popup_id: Id) -> Self {
        Self {
            editor_id,
            popup_id,
        }
    }

    /// Open or refresh suggestions at the editor cursor anchor.
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
        state.anchor = anchor;
        state.replace_len = replace_len;
        state.core.invalidate();
        if state.open {
            let rows = items.len().clamp(1, 8) as u16;
            let anchor = Anchor::Rect {
                rect: anchor,
                side: Side::Below,
                align: CrossAlign::Start,
            };
            if cx.is_open(self.popup_id) {
                cx.reanchor_layer(self.popup_id, anchor);
                cx.resize_layer(self.popup_id, LayerSize::Fixed(48, rows.saturating_add(2)));
            } else {
                cx.open_layer(
                    self.popup_id,
                    LayerSpec::popover(self.popup_id, anchor)
                        .dismiss(Dismiss::ESC_AND_OUTSIDE)
                        .size(LayerSize::Fixed(48, rows.saturating_add(2))),
                );
            }
        } else {
            cx.close_layer(self.popup_id, None);
        }
    }

    /// Close suggestions after editor motion or commit.
    pub fn dismiss(&self, cx: &mut Cx<'_>, state: &mut CompletionState) {
        state.open = false;
        state.owner = None;
        cx.close_layer(self.popup_id, None);
    }

    /// Dismiss suggestions after the editor moves independently.
    ///
    /// The editor owns its buffer and emits its own action, so this hook is
    /// called by the composition that observes an editor-motion action.  It
    /// returns whether an open completion was actually dismissed.
    pub fn dismiss_on_editor_motion(&self, cx: &mut Cx<'_>, state: &mut CompletionState) -> bool {
        if !state.open {
            return false;
        }
        self.dismiss(cx, state);
        true
    }

    /// Splice and dismiss one accepted semantic item as one lifecycle step.
    ///
    /// The replacement length belongs to [`CompletionState`], so callers
    /// cannot accidentally splice one range and dismiss a different request.
    /// Dismissal is performed even when the buffer rejects an empty insertion;
    /// the semantic item was still accepted and the popup must not remain
    /// live after that decision.
    pub fn accept(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        buffer: &mut TextBuffer,
        item: Item<'_>,
    ) -> bool {
        let inserted = Self::splice(buffer, state.replace_len, item);
        self.dismiss(cx, state);
        inserted
    }

    /// Splice one semantic item's insertion text into an editor buffer.
    fn splice(buffer: &mut TextBuffer, replace_len: usize, item: Item<'_>) -> bool {
        let end = buffer.cursor_offset();
        let start = end.saturating_sub(replace_len);
        buffer.select_range(start, end);
        buffer.insert_str(item.insertion())
    }
}

/// A semantic completion popover. The owner editor keeps focus.
///
/// ## Construction
/// `Completion::new(popup_id)`; pair it with
/// `CompletionController::new(editor_id, popup_id)`. Items arrive per phase
/// and implement [`AsItem`].
///
/// ## Ownership
/// Caller owns items and [`CompletionState`]; [`CompletionController`] owns editor/popover coordination.
///
/// ## Configuration
/// `.row`, `.empty`, `.max_rows`, `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::COMPLETION`, `DEFAULT`.
///
/// ## States
/// Cursor and disabled row states are semantic; the editor retains runtime focus.
///
/// ## Actions
/// [`CompletionAction::Accepted`] carries the semantic key; no display index escapes.
///
/// ## Focus
/// Registers no focus stop. While open, bindings are published for the
/// focused editor id stored by [`CompletionController`].
///
/// ## Keyboard
/// [`Completion::update_for`] reads editor-addressed arrows/Ctrl+N/P/page
/// keys, Tab/Enter, and Esc while layer events remain popup-addressed.
///
/// ## Mouse
/// Row click accepts; wheel and scrollbar preserve cursor independence.
///
/// ## Layout
/// Anchored popover, `24..48` columns, up to eight rows by default.
///
/// ## Parts
/// `CONTAINER`, `BORDER`, `GUTTER`, `ICON`, `ROW`, `LABEL`, `META`, `TRACK`, `THUMB`, `EMPTY`.
///
/// ## Overrides
/// Standard patch, per-part patch and single-part slot overrides.
///
/// ## Identity
/// [`AsItem`] is mandatory; insertion text is independent from visible label.
///
/// ## Testing
/// `CompletionCase`; insertion fallback and distinct insertion are unit-tested.
///
/// ## Invariants
/// Drawing never moves the cursor or ensures it visible; only cursor motion does.
pub struct Completion<'a, T, R = ItemRow> {
    id: Id,
    row: R,
    empty: Option<EmptyState<'a>>,
    max_rows: u16,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: Overrides<'a>,
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
    /// Construct a semantic completion popover.
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            row: ItemRow,
            empty: None,
            max_rows: 8,
            patch: None,
            parts: &[],
            ov: Overrides::new(),
            _item: PhantomData,
        }
    }
}

impl<'a, T, R> Completion<'a, T, R> {
    /// Styled parts.
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
    /// Component id.
    pub const fn id(&self) -> Id {
        self.id
    }
    /// Replace row painting.
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
    /// Empty state.
    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    /// Maximum visible rows.
    #[must_use]
    pub const fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = if rows == 0 { 1 } else { rows };
        self
    }
    /// Patch every part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.patch(patch);
        self
    }
    /// Patch selected parts.
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self.ov = self.ov.patch_part(parts);
        self
    }
    /// Replace one part painter.
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
            // Text and paste intents remain available to the editor so it can
            // refresh completion items after a content change.
            _ => {}
        }
    }

    fn row_is_pressed(&self, ui: &Ui<'_>, key: ItemKey) -> bool {
        Self::pressed_target(FrameRead::pressed_part(ui, self.id), key)
    }

    fn pressed_target(pressed_part: Option<PartRef>, key: ItemKey) -> bool {
        pressed_part == Some(PartRef::item(Part::ROW, key))
    }

    /// Requested popover size.
    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        let width = items
            .iter()
            .map(|item| {
                crate::text::width(item.as_item().label)
                    .saturating_add(crate::text::width(item.as_item().detail))
                    .saturating_add(8)
            })
            .max()
            .unwrap_or(24)
            .clamp(24, 48)
            .min(cx.design().size.popup_max_width);
        let rows = items
            .len()
            .min(usize::from(self.max_rows))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        LayerSize::Fixed(width, rows.saturating_add(2))
    }

    /// Update navigation and activation for the editor that retains focus.
    ///
    /// Binding intents are read from `owner`; layer and pointer intents remain
    /// addressed to this completion's popover id.
    pub fn update_for(
        &self,
        owner: Id,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
        items: &[T],
    ) -> Response<CompletionAction> {
        if !state.open {
            return Response::ignored();
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
                Intent::Layer(LayerEvent::Dismissed(_)) => {
                    state.open = false;
                    state.owner = None;
                    acc.action(CompletionAction::Dismissed);
                }
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

    /// Draw into the resolved popover area supplied by the owner's layer closure.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CompletionState, items: &[T]) -> Rect {
        ui.with_surface(crate::theme::Surface::Popover, |ui| {
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
            let inner = ui.frame(area, border.style);
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
                let mut flags = StateFlags::empty();
                if state.core.cursor() == Some(semantic.key) {
                    flags |= StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE;
                }
                if semantic.disabled {
                    flags |= StateFlags::DISABLED;
                } else if self.row_is_pressed(ui, semantic.key) {
                    flags |= StateFlags::PRESSED;
                }
                let row = Rect {
                    x: content.x,
                    y: content
                        .y
                        .saturating_add(row_i.min(usize::from(u16::MAX)) as u16),
                    width: content.width,
                    height: 1,
                };
                let mut row_ui = RowUi::new(
                    ui,
                    self.id,
                    Family::COMPLETION,
                    Variant::DEFAULT,
                    flags,
                    semantic.key,
                    row,
                );
                self.row.row(item, &mut row_ui);
                ui.register_part(self.id, PartRef::item(Part::ROW, semantic.key), row);
            }
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
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::components::{CodeEditor, CodeEditorState};
    use crate::event::Input;
    use crate::keymap::KeyMap;
    use crate::runtime::stub::key;
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;

    const EDITOR: Id = Id::root("completion.tests.editor");
    const POPUP: Id = Id::root("completion.tests.popup");
    const AREA: Rect = Rect::new(0, 0, 80, 24);
    const FIRST: ItemKey = ItemKey::num(1);
    const SECOND: ItemKey = ItemKey::num(2);
    const ITEMS: &[Item<'static>] = &[Item::new(FIRST, "alpha"), Item::new(SECOND, "beta")];

    struct RoutingApp {
        editor: CodeEditorState,
        completion: CompletionState,
        keymap: KeyMap,
        open_next: bool,
        accepted: Option<ItemKey>,
        dismissed: bool,
    }

    impl App for RoutingApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if self.open_next {
                CompletionController::new(EDITOR, POPUP).request(
                    cx,
                    &mut self.completion,
                    Rect::new(2, 1, 1, 1),
                    0,
                    ITEMS,
                );
                self.open_next = false;
            }
            let completion = Completion::<Item<'_>>::new(POPUP).update_for(
                EDITOR,
                cx,
                &mut self.completion,
                ITEMS,
            );
            match completion.action_ref() {
                Some(CompletionAction::Accepted(key)) => self.accepted = Some(*key),
                Some(CompletionAction::Dismissed) => self.dismissed = true,
                Some(CompletionAction::Moved) | None => {}
            }
            let editor = CodeEditor::new(EDITOR, 4).update(cx, &mut self.editor);
            completion.erase() | editor.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            CodeEditor::new(EDITOR, 4).draw(ui, Rect::new(0, 0, 60, 4), &self.editor);
            if self.completion.is_open() {
                let _ = ui.layer(POPUP, |ui, area| {
                    Completion::<Item<'_>>::new(POPUP).draw(ui, area, &self.completion, ITEMS);
                });
            }
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    fn routing_runtime() -> (Runtime<RoutingApp>, Buffer) {
        let mut runtime = Runtime::new(
            RoutingApp {
                editor: CodeEditorState::new("alpha\nbeta"),
                completion: CompletionState::default(),
                keymap: KeyMap::new(),
                open_next: false,
                accepted: None,
                dismissed: false,
            },
            Theme::junie(),
        );
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.set_focus(Some(EDITOR));
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Char('i')));
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.app_mut().open_next = true;
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(AREA, &mut buffer);
        (runtime, buffer)
    }

    struct ControllerApp {
        controller: CompletionController,
        completion: CompletionState,
        buffer: TextBuffer,
        request: bool,
        accept: bool,
        editor_motion: bool,
    }

    impl App for ControllerApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if self.request {
                self.controller
                    .request(cx, &mut self.completion, Rect::new(2, 1, 1, 1), 3, ITEMS);
                self.request = false;
            }
            if self.accept {
                let _ =
                    self.controller
                        .accept(cx, &mut self.completion, &mut self.buffer, ITEMS[0]);
                self.accept = false;
            }
            if self.editor_motion {
                self.controller
                    .dismiss_on_editor_motion(cx, &mut self.completion);
                self.editor_motion = false;
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            if self.completion.is_open() {
                let completion = Completion::<Item<'_>>::new(POPUP);
                let _ = ui.layer(POPUP, |ui, area| {
                    completion.draw(ui, area, &self.completion, ITEMS);
                });
            }
        }
    }

    fn controller_runtime() -> Runtime<ControllerApp> {
        Runtime::new(
            ControllerApp {
                controller: CompletionController::new(EDITOR, POPUP),
                completion: CompletionState::default(),
                buffer: TextBuffer::single("SEL cou"),
                request: false,
                accept: false,
                editor_motion: false,
            },
            Theme::junie(),
        )
    }

    #[test]
    fn completion_controller_accept_splices_and_dismisses_atomically() {
        let mut runtime = controller_runtime();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.app_mut().request = true;
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(AREA, &mut buffer);
        assert!(runtime.app().completion.is_open());
        assert!(runtime.is_open(POPUP));

        runtime.app_mut().accept = true;
        let _ = runtime.handle(Input::Tick);

        assert_eq!(runtime.app().buffer.text(), "SEL alpha");
        assert!(!runtime.app().completion.is_open());
        assert!(!runtime.is_open(POPUP));
    }

    #[test]
    fn completion_controller_dismisses_on_editor_motion() {
        let mut runtime = controller_runtime();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        runtime.app_mut().request = true;
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(AREA, &mut buffer);
        assert!(runtime.is_open(POPUP));

        runtime.app_mut().editor_motion = true;
        let _ = runtime.handle(Input::Tick);

        assert!(!runtime.app().completion.is_open());
        assert!(!runtime.is_open(POPUP));
    }

    #[test]
    fn distinct_insert_text_is_spliced() {
        let mut buffer = TextBuffer::single("SEL cou");
        let item = Item::new(ItemKey::num(1), "count").insert("COUNT(");
        assert!(CompletionController::splice(&mut buffer, 3, item));
        assert_eq!(buffer.text(), "SEL COUNT(");
    }

    #[test]
    fn none_insert_uses_label() {
        let mut buffer = TextBuffer::single("SEL cou");
        let item = Item::new(ItemKey::num(1), "count");
        assert!(CompletionController::splice(&mut buffer, 3, item));
        assert_eq!(buffer.text(), "SEL count");
    }

    #[test]
    fn focused_editor_down_moves_completion_not_editor() {
        let (mut runtime, mut buffer) = routing_runtime();
        let editor_offset = runtime.app().editor.cursor_offset();
        let _ = runtime.handle(key(KeyCode::Down));
        runtime.draw_buffer(AREA, &mut buffer);
        assert_eq!(runtime.app().completion.cursor(), Some(SECOND));
        assert_eq!(runtime.app().editor.cursor_offset(), editor_offset);
    }

    #[test]
    fn focused_editor_tab_and_enter_accept_completion() {
        for code in [KeyCode::Tab, KeyCode::Enter] {
            let (mut runtime, _) = routing_runtime();
            let _ = runtime.handle(key(code));
            assert_eq!(runtime.app().accepted, Some(FIRST));
        }
    }

    #[test]
    fn focused_editor_escape_dismisses_completion() {
        let (mut runtime, _) = routing_runtime();
        let _ = runtime.handle(key(KeyCode::Esc));
        assert!(runtime.app().dismissed);
        assert!(!runtime.app().completion.is_open());
    }

    #[test]
    fn ordinary_text_remains_owned_by_the_editor() {
        let (mut runtime, _) = routing_runtime();
        let _ = runtime.handle(key(KeyCode::Char('x')));
        assert_eq!(runtime.app().editor.text(), "xalpha\nbeta");
        assert_eq!(runtime.app().completion.cursor(), Some(FIRST));
    }

    #[test]
    fn editor_owned_completion_binding_honours_remap_and_remove() {
        let (mut runtime, mut buffer) = routing_runtime();
        runtime
            .app_mut()
            .keymap
            .remap_component(EDITOR, MOVE_DOWN, Chord::key(KeyCode::F(4)));
        let _ = runtime.handle(key(KeyCode::Down));
        assert_eq!(runtime.app().completion.cursor(), Some(FIRST));
        let _ = runtime.handle(key(KeyCode::F(4)));
        assert_eq!(runtime.app().completion.cursor(), Some(SECOND));

        runtime.draw_buffer(AREA, &mut buffer);
        runtime.app_mut().keymap.remove_component(EDITOR, MOVE_DOWN);
        let _ = runtime.handle(key(KeyCode::Up));
        let _ = runtime.handle(key(KeyCode::F(4)));
        assert_eq!(runtime.app().completion.cursor(), Some(FIRST));
    }

    #[test]
    fn pressed_part_targets_only_its_popup_row() {
        let pressed = Some(PartRef::item(Part::ROW, SECOND));
        assert!(!Completion::<Item<'static>>::pressed_target(pressed, FIRST,));
        assert!(Completion::<Item<'static>>::pressed_target(pressed, SECOND,));
    }
}
