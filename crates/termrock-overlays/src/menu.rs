use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket, shift};
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::ChordCase;
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{
    Anchor, CrossAlign, Dismiss, DismissReason, LayerEvent, LayerSize, LayerSpec, Side,
};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_feedback::keyhint::ChordText;

const MENU_BAR_TITLE_VARIANT: Variant = Variant::custom("termrock.menu-bar-title");

/// Controls whether pointer hover styles the whole menu bar or only its titles.
///
/// `Bar` preserves the standard menu component behavior. `Title` resolves
/// title styling through the theme's `termrock.menu-bar-title` variant and
/// keeps the surrounding bar unchanged during hover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuBarHoverScope {
    #[default]
    Bar,
    Title,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MenuItem<'a> {
    action: ActionKey,
    label: &'a str,
    chord: Option<Chord>,
    hint: Option<&'a str>,
    reason: Option<&'a str>,
    disabled: bool,
    danger: bool,
    separator_after: bool,
    submenu: Option<&'a [MenuItem<'a>]>,
}

impl<'a> MenuItem<'a> {
    pub const fn new(action: ActionKey, label: &'a str) -> Self {
        MenuItem {
            action,
            label,
            chord: None,
            hint: None,
            reason: None,
            disabled: false,
            danger: false,
            separator_after: false,
            submenu: None,
        }
    }

    #[must_use]
    pub const fn chord(mut self, chord: Chord) -> Self {
        self.chord = Some(chord);
        self
    }

    /// Adds display-only help text. The text is not registered as a key binding.
    #[must_use]
    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Associates explanatory text with an unavailable action.
    ///
    /// The text is displayed in the menu's bottom border when the item is
    /// hovered or highlighted. It is not a shortcut or a binding.
    #[must_use]
    pub const fn reason(mut self, reason: &'a str) -> Self {
        self.reason = Some(reason);
        self.disabled = true;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    #[must_use]
    pub const fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }

    #[must_use]
    pub const fn submenu(mut self, items: &'a [MenuItem<'a>]) -> Self {
        self.submenu = Some(items);
        self
    }

    pub const fn action(&self) -> ActionKey {
        self.action
    }

    pub const fn label(&self) -> &'a str {
        self.label
    }

    pub const fn chord_ref(&self) -> Option<Chord> {
        self.chord
    }

    pub const fn hint_ref(&self) -> Option<&'a str> {
        self.hint
    }

    pub const fn reason_ref(&self) -> Option<&'a str> {
        self.reason
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn submenu_ref(&self) -> Option<&'a [MenuItem<'a>]> {
        self.submenu
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Menu<'a> {
    pub label: &'a str,
    pub items: &'a [MenuItem<'a>],
}

impl<'a> Menu<'a> {
    pub const fn new(label: &'a str, items: &'a [MenuItem<'a>]) -> Self {
        Menu { label, items }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MenuState {
    cursor: usize,
    open: Option<usize>,
    pointed: Option<usize>,
    last_hovered: Option<PartRef>,
    hover_suppressed: bool,
    highlight_hidden: bool,
    initialized: bool,
}

impl MenuState {
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub const fn open_menu(&self) -> Option<usize> {
        self.open
    }

    pub const fn is_open(&self) -> bool {
        self.open.is_some()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    Chosen(ActionKey),
    Opened(usize),
    Closed(DismissReason),
    Submenu(ItemKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuCmd {
    Prev,
    Next,
    First,
    Last,
    Activate,
    PrevMenu,
    NextMenu,
}

const fn binding(action: ActionKey, chord: Chord, cmd: MenuCmd, visible: bool) -> Binding<MenuCmd> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label: match cmd {
            MenuCmd::Activate => "Choose",
            MenuCmd::Prev | MenuCmd::Next | MenuCmd::First | MenuCmd::Last => "Navigate",
            MenuCmd::PrevMenu | MenuCmd::NextMenu => "Menu",
        },
        priority: if visible { 70 } else { 10 },
        visible,
    }
}

const CONTEXT_BINDINGS: &[Binding<MenuCmd>] = &[
    binding(
        ActionKey::custom("menu.context.prev.up"),
        Chord::key(KeyCode::Up),
        MenuCmd::Prev,
        true,
    ),
    binding(
        ActionKey::custom("menu.context.prev.k"),
        Chord::key(KeyCode::Char('k')),
        MenuCmd::Prev,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.next.down"),
        Chord::key(KeyCode::Down),
        MenuCmd::Next,
        true,
    ),
    binding(
        ActionKey::custom("menu.context.next.j"),
        Chord::key(KeyCode::Char('j')),
        MenuCmd::Next,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.first"),
        Chord::key(KeyCode::Home),
        MenuCmd::First,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.last"),
        Chord::key(KeyCode::End),
        MenuCmd::Last,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.activate.enter"),
        Chord::key(KeyCode::Enter),
        MenuCmd::Activate,
        true,
    ),
    binding(
        ActionKey::custom("menu.context.activate.space"),
        Chord::key(KeyCode::Char(' ')),
        MenuCmd::Activate,
        false,
    ),
];

const BAR_BINDINGS: &[Binding<MenuCmd>] = &[
    binding(
        ActionKey::custom("menu.bar.prev.left"),
        Chord::key(KeyCode::Left),
        MenuCmd::PrevMenu,
        true,
    ),
    binding(
        ActionKey::custom("menu.bar.prev.h"),
        Chord::key(KeyCode::Char('h')),
        MenuCmd::PrevMenu,
        false,
    ),
    binding(
        ActionKey::custom("menu.bar.next.right"),
        Chord::key(KeyCode::Right),
        MenuCmd::NextMenu,
        true,
    ),
    binding(
        ActionKey::custom("menu.bar.next.l"),
        Chord::key(KeyCode::Char('l')),
        MenuCmd::NextMenu,
        false,
    ),
    binding(
        ActionKey::custom("menu.bar.activate.down"),
        Chord::key(KeyCode::Down),
        MenuCmd::Activate,
        true,
    ),
    binding(
        ActionKey::custom("menu.bar.activate.enter"),
        Chord::key(KeyCode::Enter),
        MenuCmd::Activate,
        false,
    ),
    binding(
        ActionKey::custom("menu.bar.activate.space"),
        Chord::key(KeyCode::Char(' ')),
        MenuCmd::Activate,
        false,
    ),
];

const BAR_CONTEXT_BINDINGS: &[Binding<MenuCmd>] = &[
    binding(
        ActionKey::custom("menu.context.prev.up"),
        Chord::key(KeyCode::Up),
        MenuCmd::Prev,
        true,
    ),
    binding(
        ActionKey::custom("menu.context.prev.k"),
        Chord::key(KeyCode::Char('k')),
        MenuCmd::Prev,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.next.down"),
        Chord::key(KeyCode::Down),
        MenuCmd::Next,
        true,
    ),
    binding(
        ActionKey::custom("menu.context.next.j"),
        Chord::key(KeyCode::Char('j')),
        MenuCmd::Next,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.first"),
        Chord::key(KeyCode::Home),
        MenuCmd::First,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.last"),
        Chord::key(KeyCode::End),
        MenuCmd::Last,
        false,
    ),
    binding(
        ActionKey::custom("menu.context.activate.enter"),
        Chord::key(KeyCode::Enter),
        MenuCmd::Activate,
        true,
    ),
    binding(
        ActionKey::custom("menu.context.activate.space"),
        Chord::key(KeyCode::Char(' ')),
        MenuCmd::Activate,
        false,
    ),
    binding(
        ActionKey::custom("menu.bar.prev.left"),
        Chord::key(KeyCode::Left),
        MenuCmd::PrevMenu,
        true,
    ),
    binding(
        ActionKey::custom("menu.bar.prev.h"),
        Chord::key(KeyCode::Char('h')),
        MenuCmd::PrevMenu,
        false,
    ),
    binding(
        ActionKey::custom("menu.bar.next.right"),
        Chord::key(KeyCode::Right),
        MenuCmd::NextMenu,
        true,
    ),
    binding(
        ActionKey::custom("menu.bar.next.l"),
        Chord::key(KeyCode::Char('l')),
        MenuCmd::NextMenu,
        false,
    ),
];

pub struct ContextMenu<'a> {
    id: Id,
    items: &'a [MenuItem<'a>],
    anchor: Anchor,
    part_anchor: Option<(Id, PartRef, Side, CrossAlign)>,
    title: Option<&'a str>,
    ov: PartStyle<'a>,
    chord_case: ChordCase,
    bar_navigation: Option<(usize, usize)>,
    retained_cursor: Option<Id>,
    skip_disabled: bool,
    clear_on_leave: bool,
}

impl fmt::Debug for ContextMenu<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextMenu")
            .field("id", &self.id)
            .field("items", &self.items.len())
            .field("anchor", &self.anchor)
            .field("part_anchor", &self.part_anchor)
            .field("title", &self.title)
            .field("bar_navigation", &self.bar_navigation)
            .field("retained_cursor", &self.retained_cursor)
            .field("skip_disabled", &self.skip_disabled)
            .field("clear_on_leave", &self.clear_on_leave)
            .finish_non_exhaustive()
    }
}

impl<'a> ContextMenu<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::TITLE,
        Part::ROW,
        Part::LABEL,
        Part::KEY,
        Part::MARKER,
        Part::RULE,
        Part::HELP,
    ];

    pub const fn new(id: Id, items: &'a [MenuItem<'a>], anchor: Anchor) -> Self {
        ContextMenu {
            id,
            items,
            anchor,
            part_anchor: None,
            title: None,
            ov: PartStyle::new(),
            chord_case: ChordCase::Preserve,
            bar_navigation: None,
            retained_cursor: None,
            skip_disabled: true,
            clear_on_leave: false,
        }
    }

    pub const fn at(id: Id, items: &'a [MenuItem<'a>], position: Position) -> Self {
        Self::new(id, items, Anchor::Point(position))
    }

    /// Anchors this menu to a part registered by another rendered component.
    /// Use [`Self::try_layer`] when opening it; that method returns `None` when
    /// the owner or part is absent from the current frame.
    #[must_use]
    pub const fn anchor_to_part(
        mut self,
        owner: Id,
        part: PartRef,
        side: Side,
        align: CrossAlign,
    ) -> Self {
        self.part_anchor = Some((owner, part, side, align));
        self
    }

    #[must_use]
    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    #[must_use]
    pub const fn chord_case(mut self, case: ChordCase) -> Self {
        self.chord_case = case;
        self
    }

    /// Controls whether keyboard navigation skips disabled items.
    /// Defaults to `true` for compatibility with existing menus.
    #[must_use]
    pub const fn skip_disabled(mut self, skip: bool) -> Self {
        self.skip_disabled = skip;
        self
    }

    /// Hides the active row highlight after the pointer leaves the menu.
    /// Keyboard navigation or a subsequent pointer move restores it.
    /// Defaults to `false` for compatibility with existing menus.
    #[must_use]
    pub const fn clear_on_leave(mut self, clear: bool) -> Self {
        self.clear_on_leave = clear;
        self
    }

    /// Keeps a real rendered owner's visible caret through this nonmodal
    /// context menu while the menu owns keyboard focus.
    #[must_use]
    pub const fn retain_cursor(mut self, owner: Id) -> Self {
        self.retained_cursor = Some(owner);
        self
    }

    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(patches);
        self
    }

    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    pub fn measured_size(&self, cx: &Cx<'_>) -> LayerSize {
        let d = cx.design();
        let width = self
            .natural_width(|action, chord| cx.effective_chord(self.id, action, chord))
            .clamp(d.size.popup_min_width, d.size.popup_max_width);
        LayerSize::Fixed(width, self.natural_height().max(3))
    }

    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        let anchor = self.resolve_anchor(cx).unwrap_or(self.anchor);
        LayerSpec::popover(self.id, anchor)
            .dismiss(Dismiss::ALL)
            .initial_focus(self.id)
            .size(self.measured_size(cx))
    }

    /// Builds a layer only when its configured owner part is present.
    /// Fixed point and rectangle anchors retain the compatibility behavior of
    /// [`Self::layer`].
    pub fn try_layer(&self, cx: &Cx<'_>) -> Option<LayerSpec> {
        let anchor = self.resolve_anchor(cx)?;
        Some(
            LayerSpec::popover(self.id, anchor)
                .dismiss(Dismiss::ALL)
                .initial_focus(self.id)
                .size(self.measured_size(cx)),
        )
    }

    fn resolve_anchor(&self, cx: &Cx<'_>) -> Option<Anchor> {
        let Some((owner, part, side, align)) = self.part_anchor else {
            return Some(self.anchor);
        };
        FrameRead::area_of_part(cx, owner, part).map(|rect| Anchor::Rect { rect, side, align })
    }

    fn natural_width(&self, effective: impl Fn(ActionKey, Option<Chord>) -> Option<Chord>) -> u16 {
        let title = self.title.map_or(0, width);
        let rows = self.items.iter().map(|item| {
            let label_width = width(item.label);
            let row_width = if let Some(hint) = item.hint {
                label_width.saturating_add(width(hint)).saturating_add(3)
            } else if let Some(chord) = effective(item.action, item.chord) {
                label_width
                    .saturating_add(width(ChordText::with_case(chord, self.chord_case).as_str()))
                    .saturating_add(3)
            } else {
                label_width
            };
            row_width.saturating_add(u16::from(item.submenu.is_some()))
        });
        rows.max().unwrap_or(4).max(title).saturating_add(6).max(8)
    }

    fn natural_height(&self) -> u16 {
        let rows = self.items.len().min(usize::from(u16::MAX)) as u16;
        let separators = self
            .items
            .iter()
            .filter(|item| item.separator_after)
            .count()
            .min(usize::from(u16::MAX)) as u16;
        rows.saturating_add(separators)
            .saturating_add(u16::from(self.title.is_some()))
            .saturating_add(2)
    }

    fn enabled_at(&self, index: usize) -> bool {
        self.items.get(index).is_some_and(|item| !item.disabled)
    }

    fn initial_cursor(&self) -> usize {
        self.first_enabled().unwrap_or(0)
    }

    fn navigation_cursor(&self, first: bool) -> Option<usize> {
        if !self.skip_disabled {
            (!self.items.is_empty()).then_some(if first {
                0
            } else {
                self.items.len().saturating_sub(1)
            })
        } else if first {
            self.first_enabled()
        } else {
            self.last_enabled()
        }
    }

    fn observe_hover(&self, st: &mut MenuState, hovered: Option<PartRef>) {
        if hovered == st.last_hovered {
            return;
        }
        if self.clear_on_leave
            && let Some(index) = st.last_hovered.and_then(Self::row_index)
            && self.enabled_at(index)
        {
            st.highlight_hidden = true;
        }
        st.last_hovered = hovered;
        st.hover_suppressed = false;
        match hovered.and_then(Self::row_index) {
            Some(index) if self.enabled_at(index) => {
                st.pointed = None;
                st.cursor = index;
                st.highlight_hidden = false;
            }
            Some(index) => st.pointed = Some(index),
            None => st.pointed = None,
        }
    }

    fn row_index(part: PartRef) -> Option<usize> {
        match part {
            PartRef {
                part: Part::ROW,
                item: Some(ItemKey::Index(index)),
            } => Some(index),
            _ => None,
        }
    }

    fn clear_pointer_state(&self, st: &mut MenuState) {
        st.pointed = None;
        st.last_hovered = None;
        st.hover_suppressed = false;
        st.highlight_hidden = false;
    }

    fn status_reason(&self, st: &MenuState) -> Option<&'a str> {
        let pointed = st.pointed.and_then(|index| self.items.get(index));
        let item = pointed.or_else(|| {
            (!st.highlight_hidden)
                .then(|| self.items.get(st.cursor))
                .flatten()
        })?;
        (item.disabled)
            .then_some(item.reason?)
            .filter(|reason| !reason.is_empty())
    }

    fn first_enabled(&self) -> Option<usize> {
        self.items.iter().position(|item| !item.disabled)
    }

    fn last_enabled(&self) -> Option<usize> {
        self.items.iter().rposition(|item| !item.disabled)
    }

    fn step(&self, st: &mut MenuState, delta: isize) -> bool {
        let len = self.items.len();
        if len == 0 {
            return false;
        }
        let mut index = st.cursor.min(len.saturating_sub(1));
        for _ in 0..len {
            index = index
                .wrapping_add_signed(delta)
                .checked_rem(len)
                .unwrap_or_default();
            if !self.skip_disabled || self.enabled_at(index) {
                let changed = st.cursor != index;
                st.cursor = index;
                return changed;
            }
        }
        false
    }

    fn activate(&self, st: &mut MenuState) -> Option<MenuAction> {
        let item = self.items.get(st.cursor).filter(|item| !item.disabled)?;
        Some(if item.submenu.is_some() {
            MenuAction::Submenu(ItemKey::index(st.cursor))
        } else {
            MenuAction::Chosen(item.action)
        })
    }

    fn command(&self, st: &mut MenuState, command: MenuCmd) -> Response<MenuAction> {
        match command {
            MenuCmd::Prev => moved(self.step(st, -1)),
            MenuCmd::Next => moved(self.step(st, 1)),
            MenuCmd::First => set_cursor(st, self.navigation_cursor(true)),
            MenuCmd::Last => set_cursor(st, self.navigation_cursor(false)),
            MenuCmd::Activate => self
                .activate(st)
                .map_or_else(Response::consumed, Response::action),
            MenuCmd::PrevMenu | MenuCmd::NextMenu => Response::ignored(),
        }
    }

    fn switch_menu(&self, delta: isize) -> Response<MenuAction> {
        let Some((index, count)) = self.bar_navigation else {
            return Response::ignored();
        };
        let next = index
            .wrapping_add_signed(delta)
            .checked_rem(count)
            .unwrap_or_default();
        Response::action(MenuAction::Opened(next))
    }

    fn binding_response(&self, st: &mut MenuState, action: ActionKey) -> Response<MenuAction> {
        if self.bar_navigation.is_some()
            && let Some(command) = Binding::command(BAR_BINDINGS, action)
        {
            return match command {
                MenuCmd::PrevMenu => self.switch_menu(-1),
                MenuCmd::NextMenu => self.switch_menu(1),
                command => self.command(st, command),
            };
        }
        Binding::command(CONTEXT_BINDINGS, action).map_or_else(
            || {
                self.items
                    .iter()
                    .find(|item| !item.disabled && item.action == action)
                    .map_or_else(Response::ignored, |item| {
                        Response::action(MenuAction::Chosen(item.action))
                    })
            },
            |command| self.command(st, command),
        )
    }

    fn is_navigation_binding(&self, action: ActionKey) -> bool {
        let command = if self.bar_navigation.is_some() {
            Binding::command(BAR_BINDINGS, action)
                .or_else(|| Binding::command(CONTEXT_BINDINGS, action))
        } else {
            Binding::command(CONTEXT_BINDINGS, action)
        };
        matches!(
            command,
            Some(MenuCmd::Prev | MenuCmd::Next | MenuCmd::First | MenuCmd::Last)
        )
    }

    fn after_keyboard_navigation(&self, st: &mut MenuState, cx: &Cx<'_>) {
        st.pointed = None;
        st.last_hovered = FrameRead::hovered_part(cx, self.id);
        st.hover_suppressed = true;
        st.highlight_hidden = false;
    }

    fn pointer(&self, st: &mut MenuState, index: usize, phase: Phase) -> Response<MenuAction> {
        if !self.enabled_at(index) {
            if phase == Phase::Move {
                st.pointed = Some(index);
                st.last_hovered = Some(PartRef::item(Part::ROW, ItemKey::index(index)));
                st.hover_suppressed = false;
            }
            return Response::consumed();
        }
        match phase {
            Phase::Click | Phase::DoubleClick => {
                st.cursor = index;
                st.pointed = None;
                st.highlight_hidden = false;
                self.activate(st)
                    .map_or_else(Response::consumed, Response::action)
            }
            Phase::Move => {
                st.last_hovered = Some(PartRef::item(Part::ROW, ItemKey::index(index)));
                st.hover_suppressed = false;
                st.pointed = None;
                st.highlight_hidden = false;
                if st.cursor == index {
                    return Response::consumed();
                }
                st.cursor = index;
                Response::changed()
            }
            _ => Response::consumed(),
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut MenuState) -> Response<MenuAction> {
        if !self.skip_disabled && !st.initialized {
            st.cursor = self.initial_cursor();
            st.initialized = true;
        } else if self.skip_disabled
            && !self.enabled_at(st.cursor)
            && let Some(first) = self.first_enabled()
        {
            st.cursor = first;
        }
        self.observe_hover(st, FrameRead::hovered_part(cx, self.id));
        if cx.is_open(self.id) {
            let Some(anchor) = self.resolve_anchor(cx) else {
                cx.close_layer(self.id, None);
                return Response::action(MenuAction::Closed(DismissReason::Programmatic));
            };
            cx.resize_layer(self.id, self.measured_size(cx));
            cx.reanchor_layer(self.id, anchor);
        }
        let mut acc = Acc::new();
        for intent in cx.intents(self.id) {
            let response = match intent {
                Intent::Layer(LayerEvent::Dismissed(reason)) => {
                    self.clear_pointer_state(st);
                    st.initialized = false;
                    Response::action(MenuAction::Closed(reason))
                }
                Intent::Layer(LayerEvent::Opened) => {
                    if !self.skip_disabled {
                        st.cursor = self.initial_cursor();
                        st.initialized = true;
                    }
                    self.clear_pointer_state(st);
                    Response::ignored().repaint()
                }
                Intent::Layer(LayerEvent::Closed(_)) => {
                    self.clear_pointer_state(st);
                    st.initialized = false;
                    Response::ignored().repaint()
                }
                Intent::Cancel => {
                    cx.close_layer(self.id, None);
                    Response::action(MenuAction::Closed(DismissReason::Esc))
                }
                Intent::Binding(action) => {
                    let navigation = self.is_navigation_binding(action);
                    let response = self.binding_response(st, action);
                    if navigation {
                        self.after_keyboard_navigation(st, cx);
                    }
                    response
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(ItemKey::Index(index)),
                        },
                    ..
                } => self.pointer(st, index, phase),
                Intent::Pointer { .. } => Response::consumed(),
                _ => Response::ignored(),
            };
            if let Some(action) = response.action_ref().copied() {
                if let MenuAction::Chosen(key) = action {
                    cx.close_layer(self.id, Some(key));
                }
                acc.action(action);
            } else {
                acc.fold(&response.erase());
            }
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &MenuState) -> Rect {
        if area.is_empty() {
            return area;
        }
        ui.with_surface(Surface::Popover, |ui| {
            let mut live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
            live.remove(StateFlags::PRESSED);
            let container = self.ov.style(
                ui,
                self.id,
                Family::MENU,
                Variant::DEFAULT,
                Part::CONTAINER,
                live,
            );
            ui.fill(area, container.style);
            let border = self.ov.style(
                ui,
                self.id,
                Family::MENU,
                Variant::DEFAULT,
                Part::BORDER,
                live,
            );
            let inner = ui.frame(area, border.style);
            if let Some(slot) = self.ov.slot_for(Part::BORDER) {
                slot(ui, area);
            }
            ui.register_control(self.id, area, Focusability::Focusable);
            let bindings = if self.bar_navigation.is_some() {
                BAR_CONTEXT_BINDINGS
            } else {
                CONTEXT_BINDINGS
            };
            ui.publish_bindings(self.id, live, bindings);
            ui.publish_dynamic_bindings(
                self.id,
                live,
                self.items
                    .iter()
                    .filter(|item| !item.disabled)
                    .map(|item| (item.action, item.chord)),
            );
            ui.register_decor(self.id, PartRef::of(Part::BORDER), area);
            let mut y = inner.y;
            if let Some(title) = self.title {
                let row = shift(first_row(Rect { y, ..inner }), 2);
                let row = Rect {
                    width: row.width.saturating_sub(1),
                    ..row
                };
                let style = self.ov.style(
                    ui,
                    self.id,
                    Family::MENU,
                    Variant::DEFAULT,
                    Part::TITLE,
                    live,
                );
                paint_or_slot(ui, &self.ov, Part::TITLE, row, title, style.style);
                ui.register_decor(self.id, PartRef::of(Part::TITLE), row);
                y = y.saturating_add(1);
            }
            for (index, item) in self.items.iter().enumerate() {
                if y >= inner.bottom() {
                    break;
                }
                let row = first_row(Rect { y, ..inner });
                self.draw_row(ui, row, index, item, st, live);
                y = y.saturating_add(1);
                if item.separator_after && y < inner.bottom() {
                    let rule = first_row(Rect { y, ..inner });
                    let style = self.ov.style(
                        ui,
                        self.id,
                        Family::MENU,
                        Variant::DEFAULT,
                        Part::RULE,
                        live,
                    );
                    if let Some(slot) = self.ov.slot_for(Part::RULE) {
                        slot(ui, rule);
                    } else {
                        for cell in rule.rows() {
                            ui.glyph(cell, GlyphRole::RuleQuiet, style.style);
                        }
                    }
                    ui.register_decor(self.id, PartRef::of(Part::RULE), rule);
                    y = y.saturating_add(1);
                }
            }
            if let Some(reason) = self.status_reason(st) {
                self.draw_reason(ui, area, reason);
            }
        });
        if let Some(owner) = self.retained_cursor {
            let _ = ui.retain_visible_owner_cursor(owner);
        }
        area
    }

    fn draw_row(
        &self,
        ui: &mut Ui<'_>,
        row: Rect,
        index: usize,
        item: &MenuItem<'_>,
        st: &MenuState,
        parent: StateFlags,
    ) {
        let mut derived = parent.difference(
            StateFlags::FOCUSED
                | StateFlags::FOCUS_VISIBLE
                | StateFlags::HOVERED
                | StateFlags::PRESSED,
        );
        if !st.hover_suppressed
            && FrameRead::hovered_part(ui, self.id)
                == Some(PartRef::item(Part::ROW, ItemKey::index(index)))
        {
            derived |= StateFlags::HOVERED;
        }
        if index == st.cursor && !st.highlight_hidden {
            derived |=
                StateFlags::ACTIVE | parent & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
        }
        if item.disabled {
            derived |= StateFlags::DISABLED;
        }
        let mut flags = PartStyle::flags(StateFlags::empty(), derived);
        let pressed_row = pressed_target(ui, self.id, Part::ROW, index);
        if pressed_row {
            flags.insert(StateFlags::PRESSED);
        } else {
            flags.remove(StateFlags::PRESSED);
        }
        let variant = if item.danger {
            Variant::DANGER
        } else {
            Variant::DEFAULT
        };
        let style = self
            .ov
            .style(ui, self.id, Family::MENU, variant, Part::ROW, flags);
        ui.fill(row, style.style);
        let marker = Rect { width: 1, ..row };
        if item.submenu.is_some() {
            let marker_style =
                self.ov
                    .style(ui, self.id, Family::MENU, variant, Part::MARKER, flags);
            if let Some(slot) = self.ov.slot_for(Part::MARKER) {
                slot(ui, marker);
            } else {
                let glyph = match marker_style.glyph {
                    Slot::Set(glyph) => glyph,
                    Slot::Inherit | Slot::Clear => GlyphRole::Expanded,
                };
                ui.glyph(marker, glyph, marker_style.style);
            }
        }
        let label = shift(row, 2);
        let effective_chord = ui.effective_chord(self.id, item.action, item.chord);
        let chord_text = effective_chord.map(|chord| ChordText::with_case(chord, self.chord_case));
        let hint = item
            .hint
            .or_else(|| chord_text.as_ref().map(|text| text.as_str()));
        let hint_width = hint.map_or(0, width);
        let hint_admitted = hint.is_some() && admits_hint(item.label, hint_width, row.width);
        let reserved_hint = if hint_admitted { hint_width } else { 0 };
        let label = Rect {
            width: if hint_admitted {
                row.width.saturating_sub(reserved_hint.saturating_add(5))
            } else {
                row.width.saturating_sub(3)
            },
            ..label
        };
        let label_style = self
            .ov
            .style(ui, self.id, Family::MENU, variant, Part::LABEL, flags);
        paint_or_slot(
            ui,
            &self.ov,
            Part::LABEL,
            label,
            item.label,
            label_style.style,
        );
        if hint_admitted && let Some(text) = hint {
            let key = Rect {
                x: row.right().saturating_sub(reserved_hint.saturating_add(1)),
                width: reserved_hint.min(row.width),
                ..row
            };
            let key_style = self
                .ov
                .style(ui, self.id, Family::MENU, variant, Part::KEY, flags);
            paint_or_slot(ui, &self.ov, Part::KEY, key, text, key_style.style);
        }
        ui.register_part(
            self.id,
            PartRef::item(Part::ROW, ItemKey::index(index)),
            row,
        );
    }

    fn draw_reason(&self, ui: &mut Ui<'_>, area: Rect, reason: &str) {
        let available = area.width.saturating_sub(4);
        if available < 3 || reason.is_empty() {
            return;
        }
        let reason = crate::text::truncate(reason, available.saturating_sub(2));
        let padded = format!(" {reason} ");
        let width = width(&padded).min(available);
        if width == 0 {
            return;
        }
        let row = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(area.height.saturating_sub(1)),
            width,
            height: 1,
        };
        let style = self.ov.style(
            ui,
            self.id,
            Family::MENU,
            Variant::DEFAULT,
            Part::HELP,
            StateFlags::empty(),
        );
        if let Some(slot) = self.ov.slot_for(Part::HELP) {
            slot(ui, row);
        } else {
            ui.paint_str(row, &padded, style.style);
        }
        ui.register_decor(self.id, PartRef::of(Part::HELP), row);
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(
            self.natural_width(|action, chord| ui.effective_chord(self.id, action, chord)),
            self.natural_height(),
        )
        .fit(c)
    }
}

impl Bindings for ContextMenu<'_> {
    type Cmd = MenuCmd;

    fn bindings(&self, _state: BindingState) -> &'static [Binding<MenuCmd>] {
        CONTEXT_BINDINGS
    }
}

pub struct MenuBar<'a> {
    id: Id,
    menus: &'a [Menu<'a>],
    ov: PartStyle<'a>,
    chord_case: ChordCase,
    hover_scope: MenuBarHoverScope,
}

impl fmt::Debug for MenuBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MenuBar")
            .field("id", &self.id)
            .field("menus", &self.menus.len())
            .field("hover_scope", &self.hover_scope)
            .finish_non_exhaustive()
    }
}

impl<'a> MenuBar<'a> {
    pub const PARTS: &'static [Part] = ContextMenu::PARTS;

    pub const fn new(id: Id, menus: &'a [Menu<'a>]) -> Self {
        MenuBar {
            id,
            menus,
            ov: PartStyle::new(),
            chord_case: ChordCase::Preserve,
            hover_scope: MenuBarHoverScope::Bar,
        }
    }

    /// Sets whether pointer hover styling applies to the whole bar or each title.
    #[must_use]
    pub const fn hover_scope(mut self, scope: MenuBarHoverScope) -> Self {
        self.hover_scope = scope;
        self
    }

    #[must_use]
    pub const fn chord_case(mut self, case: ChordCase) -> Self {
        self.chord_case = case;
        self
    }

    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(patches);
        self
    }

    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    fn menu_id(&self, index: usize) -> Id {
        self.id.item(ItemKey::index(index))
    }

    fn anchor(&self, cx: &Cx<'_>, index: usize) -> Anchor {
        cx.area(self.menu_id(index)).map_or(
            Anchor::Screen(crate::layer::ScreenAlign::UpperThird),
            |rect| Anchor::Rect {
                rect,
                side: crate::layer::Side::Below,
                align: crate::layer::CrossAlign::Start,
            },
        )
    }

    fn dropdown(&self, cx: &Cx<'_>, index: usize) -> Option<ContextMenu<'a>> {
        let menu = self.menus.get(index)?;
        Some(ContextMenu {
            id: self.id,
            items: menu.items,
            anchor: self.anchor(cx, index),
            part_anchor: None,
            title: None,
            ov: self.ov,
            chord_case: self.chord_case,
            bar_navigation: Some((index, self.menus.len())),
            retained_cursor: None,
            skip_disabled: true,
            clear_on_leave: false,
        })
    }

    fn open(&self, cx: &mut Cx<'_>, st: &mut MenuState, index: usize) -> MenuAction {
        let index = index.min(self.menus.len().saturating_sub(1));
        st.open = Some(index);
        st.cursor = self
            .menus
            .get(index)
            .and_then(|menu| menu.items.iter().position(|item| !item.disabled))
            .unwrap_or(0);
        if let Some(dropdown) = self.dropdown(cx, index) {
            cx.open_layer(self.id, dropdown.layer(cx));
        }
        MenuAction::Opened(index)
    }

    pub fn open_menu(
        &self,
        cx: &mut Cx<'_>,
        st: &mut MenuState,
        index: usize,
    ) -> Response<MenuAction> {
        if self.menus.is_empty() {
            return Response::ignored();
        }
        Response::action(self.open(cx, st, index)).for_id(self.id)
    }

    fn close_missing(&self, cx: &mut Cx<'_>, st: &mut MenuState) -> Response<MenuAction> {
        let had_open_state = st.open.take().is_some();
        let had_live_layer = cx.is_open(self.id);
        if !had_open_state && !had_live_layer {
            return Response::ignored();
        }
        cx.close_layer(self.id, None);
        Response::action(MenuAction::Closed(DismissReason::Programmatic)).for_id(self.id)
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut MenuState) -> Response<MenuAction> {
        if self.menus.is_empty() {
            let response = self.close_missing(cx, st);
            for _ in cx.intents(self.id) {}
            return response;
        }
        if let Some(open) = st.open {
            let Some(dropdown) = self.dropdown(cx, open) else {
                let response = self.close_missing(cx, st);
                for _ in cx.intents(self.id) {}
                return response;
            };
            let response = dropdown.update(cx, st);
            match response.action_ref().copied() {
                Some(MenuAction::Opened(index)) => {
                    let _ = self.open(cx, st, index);
                    if let Some(next) = self.dropdown(cx, index) {
                        cx.resize_layer(self.id, next.measured_size(cx));
                        cx.reanchor_layer(self.id, next.anchor);
                    }
                }
                Some(MenuAction::Chosen(key)) => {
                    st.open = None;
                    cx.close_layer(self.id, Some(key));
                }
                Some(MenuAction::Closed(_)) => {
                    st.open = None;
                    cx.close_layer(self.id, None);
                }
                _ => {}
            }
            return response;
        }
        let mut acc = Acc::new();
        for intent in cx.intents(self.id) {
            let action = match intent {
                Intent::Binding(action) => match Binding::command(BAR_BINDINGS, action) {
                    Some(MenuCmd::PrevMenu) => {
                        st.cursor = st
                            .cursor
                            .wrapping_sub(1)
                            .checked_rem(self.menus.len())
                            .unwrap_or_default();
                        acc.changed();
                        None
                    }
                    Some(MenuCmd::NextMenu) => {
                        st.cursor = st
                            .cursor
                            .wrapping_add(1)
                            .checked_rem(self.menus.len())
                            .unwrap_or_default();
                        acc.changed();
                        None
                    }
                    Some(MenuCmd::Activate) => Some(self.open(cx, st, st.cursor)),
                    _ => None,
                },
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part:
                        PartRef {
                            part: Part::TITLE,
                            item: Some(ItemKey::Index(index)),
                        },
                    ..
                } if index < self.menus.len() => Some(self.open(cx, st, index)),
                Intent::Pointer { .. } => {
                    acc.consumed();
                    None
                }
                _ => None,
            };
            if let Some(action) = action {
                acc.action(action);
            }
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &MenuState) -> Rect {
        let row = first_row(area);
        if row.is_empty() {
            return row;
        }
        let mut live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        live.remove(StateFlags::PRESSED);
        if self.hover_scope == MenuBarHoverScope::Title {
            live.remove(StateFlags::HOVERED);
        }
        let container = self.ov.style(
            ui,
            self.id,
            Family::MENU,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(row, container.style);
        if st.open.is_none() {
            ui.register_control(self.id, row, Focusability::Focusable);
            ui.publish_bindings(self.id, live, BAR_BINDINGS);
        }
        let mut x = row.x.saturating_add(1);
        for (index, menu) in self.menus.iter().enumerate() {
            let w = width(menu.label).saturating_add(2);
            if x.saturating_add(w) > row.right() {
                break;
            }
            let rect = Rect {
                x,
                y: row.y,
                width: w,
                height: 1,
            };
            let current = st.open == Some(index) || (st.open.is_none() && st.cursor == index);
            let mut flags = ui.state(self.menu_id(index));
            if current {
                flags |=
                    StateFlags::ACTIVE | live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            if FrameRead::hovered_part(ui, self.id)
                == Some(PartRef::item(Part::TITLE, ItemKey::index(index)))
            {
                flags |= StateFlags::HOVERED;
            }
            let pressed_title = pressed_target(ui, self.id, Part::TITLE, index);
            if pressed_title {
                flags.insert(StateFlags::PRESSED);
            } else {
                flags.remove(StateFlags::PRESSED);
            }
            let style = match self.hover_scope {
                MenuBarHoverScope::Bar => self.ov.style(
                    ui,
                    self.id,
                    Family::MENU,
                    Variant::DEFAULT,
                    Part::TITLE,
                    flags,
                ),
                MenuBarHoverScope::Title => {
                    let mut defaults = StylePatch::default();
                    if let Some(recipe) = ui
                        .theme_ref()
                        .recipes
                        .get(Family::MENU)
                        .and_then(|recipe| recipe.variant(MENU_BAR_TITLE_VARIANT))
                        .and_then(|parts| parts.get(Part::TITLE))
                    {
                        defaults = recipe.apply(defaults, flags);
                    }
                    let local = self.ov.part_patch(Part::TITLE);
                    let style = ui.style_defaults(
                        Family::MENU,
                        Variant::DEFAULT,
                        Part::TITLE,
                        flags,
                        crate::theme::StyleDefaults::new(defaults),
                        local.as_ref(),
                    );
                    self.ov.note(
                        ui,
                        self.id,
                        Family::MENU,
                        Variant::DEFAULT,
                        Part::TITLE,
                        style,
                    );
                    style
                }
            };
            if self.hover_scope == MenuBarHoverScope::Title {
                ui.fill(rect, style.style);
            }
            let label = shift(rect, 1);
            let padded = format!(" {} ", menu.label);
            if let Some(slot) = self.ov.slot_for(Part::TITLE) {
                slot(ui, label);
            } else if matches!(style.glyph, Slot::Set(GlyphRole::PressLeft)) {
                ui.paint_str(rect, &padded, style.style);
                paint_pressed_bracket(
                    ui,
                    cell_at(rect, rect.x),
                    cell_at(rect, rect.right().saturating_sub(1)),
                    style.style,
                );
            } else {
                ui.paint_str(rect, &padded, style.style);
            }
            ui.register_decor(
                self.menu_id(index),
                PartRef::item(Part::TITLE, ItemKey::index(index)),
                rect,
            );
            ui.register_part(
                self.id,
                PartRef::item(Part::TITLE, ItemKey::index(index)),
                rect,
            );
            x = rect.right().saturating_add(1);
        }
        if let Some(open) = st.open
            && let Some(menu) = self.menus.get(open)
        {
            let dropdown = ContextMenu {
                id: self.id,
                items: menu.items,
                anchor: Anchor::Screen(crate::layer::ScreenAlign::UpperThird),
                part_anchor: None,
                title: None,
                ov: self.ov,
                chord_case: self.chord_case,
                bar_navigation: Some((open, self.menus.len())),
                retained_cursor: None,
                skip_disabled: true,
                clear_on_leave: false,
            };
            let _ = ui.layer(self.id, |ui, layer| dropdown.draw(ui, layer, st));
        }
        row
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let width = self.menus.iter().fold(1u16, |total, menu| {
            total.saturating_add(width(menu.label).saturating_add(3))
        });
        Size::exact(width, 1).fit(c)
    }
}

impl Bindings for MenuBar<'_> {
    type Cmd = MenuCmd;

    fn bindings(&self, _state: BindingState) -> &'static [Binding<MenuCmd>] {
        BAR_BINDINGS
    }
}

fn paint_or_slot(
    ui: &mut Ui<'_>,
    overrides: &PartStyle<'_>,
    part: Part,
    area: Rect,
    text: &str,
    style: crate::theme::PaintStyle,
) {
    if let Some(slot) = overrides.slot_for(part) {
        slot(ui, area);
    } else {
        ui.paint_str(area, text, style);
    }
}

fn pressed_target(ui: &Ui<'_>, owner: Id, part: Part, index: usize) -> bool {
    FrameRead::pressed_part(ui, owner) == Some(PartRef::item(part, ItemKey::index(index)))
}

fn admits_hint(label: &str, hint_width: u16, row_width: u16) -> bool {
    width(label).saturating_add(hint_width).saturating_add(5) <= row_width
}

fn moved(changed: bool) -> Response<MenuAction> {
    if changed {
        Response::changed()
    } else {
        Response::consumed()
    }
}

fn set_cursor(st: &mut MenuState, index: Option<usize>) -> Response<MenuAction> {
    match index {
        Some(index) if index != st.cursor => {
            st.cursor = index;
            Response::changed()
        }
        _ => Response::consumed(),
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::event::{Input, Key, KeyModifiers, Mouse, MouseKind};
    use crate::keymap::KeyMap;
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;

    const OPEN: ActionKey = ActionKey::custom("open");
    const CLOSE: ActionKey = ActionKey::custom("close");
    const MORE: ActionKey = ActionKey::custom("more");
    const CHILD: [MenuItem<'static>; 1] = [MenuItem::new(OPEN, "Child")];
    const ITEMS: [MenuItem<'static>; 4] = [
        MenuItem::new(OPEN, "Open").chord(Chord::key(KeyCode::Char('o'))),
        MenuItem::new(CLOSE, "Disabled")
            .chord(Chord::key(KeyCode::Char('d')))
            .disabled(true),
        MenuItem::new(CLOSE, "Close").danger().separator(),
        MenuItem::new(MORE, "More").submenu(&CHILD),
    ];

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    #[test]
    fn context_menu_title_does_not_inherit_owner_hover() {
        const ID: Id = Id::root("menu.tests.context-title-hover");
        let area = Rect::new(0, 0, 24, 8);
        let render = |hovered: bool| {
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, area| {
                    ui.reference(
                        hovered.then_some(crate::ReferenceTarget::new(
                            ID,
                            crate::ReferenceState::HOVERED,
                        )),
                        |ui| {
                            ContextMenu::at(ID, &ITEMS, Position::new(0, 0))
                                .title("Heading")
                                .draw(ui, area, &MenuState::default());
                        },
                    );
                })
                .commit_presented();
            buffer
        };
        let resting = render(false);
        let hovered = render(true);
        for x in 1..23 {
            assert_eq!(
                resting.cell(Position::new(x, 1)),
                hovered.cell(Position::new(x, 1)),
                "context-menu heading cell at x={x} changed when its owner was hovered"
            );
        }
    }

    #[test]
    fn menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence() {
        const ID: Id = Id::root("menu.tests.public-title-resolution");
        const VARIANT: Variant = Variant::custom("menu.tests.public-title-variant");
        const OVERLAY_RULES: [crate::theme::OverlayRule; 1] = [(
            Family::MENU,
            VARIANT,
            Part::TITLE,
            StateFlags::HOVERED,
            StylePatch::new().set_bg(crate::theme::Role::Custom(ratatui_core::style::Color::Rgb(
                7, 8, 9,
            ))),
        )];
        const OVERLAY: crate::theme::Overlay = crate::theme::Overlay::new(&OVERLAY_RULES);
        const LOCAL: [(Part, StylePatch); 1] = [(
            Part::TITLE,
            StylePatch::new().set_fg(crate::theme::Role::Custom(ratatui_core::style::Color::Rgb(
                10, 11, 12,
            ))),
        )];
        const MENUS: [Menu<'static>; 1] = [Menu::new("File", &ITEMS)];

        let theme = Theme::junie().define_variant(Family::MENU, VARIANT, |recipe| {
            recipe.default_variant(VARIANT);
            recipe.part(Part::TITLE).when(
                StateFlags::HOVERED,
                StylePatch::new().add(crate::theme::Modifier::ITALIC),
            );
        });
        let area = Rect::new(0, 0, 20, 2);
        for scope in [MenuBarHoverScope::Bar, MenuBarHoverScope::Title] {
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), theme.clone());
            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, area| {
                    ui.with_overlay(&OVERLAY, |ui| {
                        ui.reference(
                            Some(
                                crate::ReferenceTarget::new(ID, crate::ReferenceState::HOVERED)
                                    .part(PartRef::item(Part::TITLE, ItemKey::index(0))),
                            ),
                            |ui| {
                                MenuBar::new(ID, &MENUS)
                                    .hover_scope(scope)
                                    .patch_part(&LOCAL)
                                    .draw(ui, area, &MenuState::default());
                            },
                        );
                    });
                })
                .commit_presented();

            let title = buffer.cell(Position::new(2, 0)).expect("File title cell");
            assert_eq!(
                title.fg,
                ratatui_core::style::Color::Rgb(10, 11, 12),
                "foreground with {scope:?}"
            );
            assert_eq!(
                title.bg,
                ratatui_core::style::Color::Rgb(7, 8, 9),
                "overlay background with {scope:?}"
            );
            assert!(
                title.modifier.contains(crate::theme::Modifier::ITALIC),
                "family default variant with {scope:?}"
            );
        }
    }

    #[test]
    fn default_menu_bar_scope_keeps_title_patches_off_the_padding_cells() {
        const ID: Id = Id::root("menu.tests.default-title-padding");
        const MENUS: [Menu<'static>; 1] = [Menu::new("File", &ITEMS)];
        const PATCH: [(Part, StylePatch); 1] = [(
            Part::TITLE,
            StylePatch::new().set_bg(crate::theme::Role::Custom(ratatui_core::style::Color::Rgb(
                7, 8, 9,
            ))),
        )];
        let area = Rect::new(0, 0, 20, 1);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                MenuBar::new(ID, &MENUS)
                    .patch_part(&PATCH)
                    .draw(ui, area, &MenuState::default());
            })
            .commit_presented();

        let title_bg = ratatui_core::style::Color::Rgb(7, 8, 9);
        assert_eq!(buffer.cell(Position::new(2, 0)).unwrap().bg, title_bg);
        assert_ne!(buffer.cell(Position::new(1, 0)).unwrap().bg, title_bg);
        assert_ne!(buffer.cell(Position::new(6, 0)).unwrap().bg, title_bg);
    }

    #[test]
    fn context_menu_heading_honors_explicit_owner_hover_overlay() {
        const ID: Id = Id::root("menu.tests.context-title-overlay");
        const RULES: [crate::theme::OverlayRule; 1] = [(
            Family::MENU,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::HOVERED,
            StylePatch::new()
                .set_fg(crate::theme::Role::Custom(ratatui_core::style::Color::Rgb(
                    21, 22, 23,
                )))
                .set_bg(crate::theme::Role::Custom(ratatui_core::style::Color::Rgb(
                    24, 25, 26,
                ))),
        )];
        const OVERLAY: crate::theme::Overlay = crate::theme::Overlay::new(&RULES);
        let area = Rect::new(0, 0, 24, 8);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.with_overlay(&OVERLAY, |ui| {
                    ui.reference(
                        Some(crate::ReferenceTarget::new(
                            ID,
                            crate::ReferenceState::HOVERED,
                        )),
                        |ui| {
                            ContextMenu::at(ID, &ITEMS, Position::new(0, 0))
                                .title("Heading")
                                .draw(ui, area, &MenuState::default());
                        },
                    );
                });
            })
            .commit_presented();
        let heading = buffer.cell(Position::new(3, 1)).expect("heading cell");
        assert_eq!(heading.fg, ratatui_core::style::Color::Rgb(21, 22, 23));
        assert_eq!(heading.bg, ratatui_core::style::Color::Rgb(24, 25, 26));
    }

    #[test]
    fn keyboard_skips_disabled_and_wraps() {
        let menu = ContextMenu::at(Id::root("menu.tests"), &ITEMS, Position::new(1, 1));
        let mut state = MenuState::default();
        assert!(menu.step(&mut state, 1));
        assert_eq!(state.cursor(), 2);
        assert!(menu.step(&mut state, 1));
        assert_eq!(state.cursor(), 3);
        assert!(menu.step(&mut state, 1));
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn displayed_chord_is_the_handled_chord() {
        let item = &ITEMS[0];
        let chord = item.chord_ref().unwrap_or(Chord::key(KeyCode::Null));
        assert!(chord.matches(&key(KeyCode::Char('o'))));
        assert_eq!(ChordText::of(chord).as_str(), "o");
        assert_eq!(item.action(), OPEN);
    }

    #[test]
    fn display_hints_size_and_paint_without_becoming_bindings() {
        const HINT_ACTION: ActionKey = ActionKey::custom("menu.hint.only");
        const CLICK_ACTION: ActionKey = ActionKey::custom("menu.hint.pointer-only");
        const HINT_ITEMS: [MenuItem<'static>; 2] = [
            MenuItem::new(HINT_ACTION, "Split right").hint("Ctrl+B %"),
            MenuItem::new(CLICK_ACTION, "Duplicate").hint("2×click"),
        ];
        const MENU: Id = Id::root("menu.hint-only.runtime");
        let component = ContextMenu::at(MENU, &HINT_ITEMS, Position::new(0, 0));
        assert_eq!(HINT_ITEMS[0].hint_ref(), Some("Ctrl+B %"));
        assert_eq!(HINT_ITEMS[0].chord_ref(), None);
        assert_eq!(HINT_ITEMS[1].hint_ref(), Some("2×click"));
        assert_eq!(HINT_ITEMS[1].chord_ref(), None);
        let required = width("Split right") + width("Ctrl+B %") + 5;
        assert!(admits_hint("Split right", width("Ctrl+B %"), required));
        assert!(!admits_hint("Split right", width("Ctrl+B %"), required - 1));
        assert_eq!(component.natural_width(|_, chord| chord), 28);

        assert!(admits_hint(
            "壊れた",
            width("2×click"),
            width("壊れた") + width("2×click") + 5
        ));
        assert!(!admits_hint(
            "壊れた",
            width("2×click"),
            width("壊れた") + width("2×click") + 4
        ));
        let area = Rect::new(0, 0, 40, 8);
        for color in [
            crate::ColorLevel::TrueColor,
            crate::ColorLevel::Ansi256,
            crate::ColorLevel::Ansi16,
            crate::ColorLevel::Mono,
        ] {
            let mut buffer = Buffer::empty(area);
            let mut runtime = Runtime::new(
                HintMenuApp {
                    state: MenuState::default(),
                    chosen: None,
                },
                Theme::junie().downgrade(color),
            );
            let _ = runtime.initialize();
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            let painted = buffer
                .content()
                .iter()
                .map(ratatui_core::buffer::Cell::symbol)
                .collect::<String>();
            assert!(painted.contains("Ctrl+B %"), "first hint at {color:?}");
            assert!(painted.contains("2×click"), "second hint at {color:?}");

            let _ =
                crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('2'))));
            assert_eq!(
                runtime.app().chosen,
                None,
                "display hint bound key at {color:?}"
            );

            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Down,
                    pos: Position::new(31, 1),
                    mods: KeyModifiers::NONE,
                }),
            );
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Up,
                    pos: Position::new(31, 1),
                    mods: KeyModifiers::NONE,
                }),
            );
            assert_eq!(
                runtime.app().chosen,
                Some(HINT_ACTION),
                "row hit at {color:?}"
            );
        }
    }

    #[test]
    fn natural_width_does_not_reserve_a_shortcut_gutter_without_a_shortcut() {
        const PLAIN: [MenuItem<'static>; 1] = [MenuItem::new(OPEN, "0123456789")];
        const SHORTCUT: [MenuItem<'static>; 1] =
            [MenuItem::new(OPEN, "0123456789").chord(Chord::key(KeyCode::Char('o')))];
        const LONG_SHORTCUT: [MenuItem<'static>; 1] =
            [MenuItem::new(OPEN, "012345678901234567890123456789")
                .chord(Chord::key(KeyCode::Char('o')))];

        let plain = ContextMenu::at(
            Id::root("menu.tests.plain-width"),
            &PLAIN,
            Position::new(0, 0),
        );
        let shortcut = ContextMenu::at(
            Id::root("menu.tests.shortcut-width"),
            &SHORTCUT,
            Position::new(0, 0),
        );

        assert_eq!(plain.natural_width(|_, chord| chord), 16);
        assert_eq!(shortcut.natural_width(|_, chord| chord), 20);
        let long_shortcut = ContextMenu::at(
            Id::root("menu.tests.long-shortcut-width"),
            &LONG_SHORTCUT,
            Position::new(0, 0),
        );
        assert_eq!(long_shortcut.natural_width(|_, chord| chord), 40);
    }

    #[test]
    fn disabled_reasons_are_borrowed_and_do_not_change_menu_measurement() {
        const REASON: &str = "The clipboard is empty";
        const WITHOUT_REASON: [MenuItem<'static>; 2] = [
            MenuItem::new(OPEN, "Open"),
            MenuItem::new(CLOSE, "Paste").chord(Chord::key(KeyCode::Char('p'))),
        ];
        const WITH_REASON: [MenuItem<'static>; 2] = [
            MenuItem::new(OPEN, "Open"),
            MenuItem::new(CLOSE, "Paste")
                .chord(Chord::key(KeyCode::Char('p')))
                .reason(REASON),
        ];

        assert_eq!(WITH_REASON[1].reason_ref(), Some(REASON));
        assert!(WITH_REASON[1].is_disabled());
        assert_eq!(
            WITH_REASON[1].chord_ref(),
            Some(Chord::key(KeyCode::Char('p')))
        );
        assert_eq!(
            WITH_REASON[1].disabled(false).reason_ref(),
            Some(REASON),
            "explicit enabled override retains the borrowed explanation"
        );
        assert!(!WITH_REASON[1].disabled(false).is_disabled());

        let plain = ContextMenu::at(
            Id::root("menu.tests.reason-width.plain"),
            &WITHOUT_REASON,
            Position::new(0, 0),
        );
        let described = ContextMenu::at(
            Id::root("menu.tests.reason-width.described"),
            &WITH_REASON,
            Position::new(0, 0),
        );
        assert_eq!(plain.natural_width(|_, chord| chord), 15);
        assert_eq!(
            described.natural_width(|_, chord| chord),
            plain.natural_width(|_, chord| chord)
        );

        let borrowed = String::from("borrowed reason");
        let item = MenuItem::new(CLOSE, "Paste").reason(&borrowed);
        assert_eq!(item.reason_ref(), Some(borrowed.as_str()));
    }

    const REASON_OPEN: ActionKey = ActionKey::custom("menu.reason.open");
    const REASON_PASTE: ActionKey = ActionKey::custom("menu.reason.paste");
    const REASON_CLEAR: ActionKey = ActionKey::custom("menu.reason.clear");
    const REASON_ITEMS: [MenuItem<'static>; 3] = [
        MenuItem::new(REASON_OPEN, "Open"),
        MenuItem::new(REASON_PASTE, "Paste")
            .chord(Chord::key(KeyCode::Char('p')))
            .reason("The clipboard is empty"),
        MenuItem::new(REASON_CLEAR, "Clear")
            .reason("Nothing is selected")
            .separator(),
    ];
    const REASON_MENU: Id = Id::root("menu.disabled-reason.runtime");

    struct ReasonMenuApp {
        state: MenuState,
        chosen: Option<ActionKey>,
    }

    fn reason_menu() -> ContextMenu<'static> {
        ContextMenu::at(REASON_MENU, &REASON_ITEMS, Position::new(0, 0))
            .skip_disabled(false)
            .clear_on_leave(true)
    }

    impl App for ReasonMenuApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = reason_menu().update(cx, &mut self.state);
            if let Some(MenuAction::Chosen(action)) = response.action_ref() {
                self.chosen = Some(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            reason_menu().draw(ui, Rect::new(0, 0, 40, 8), &self.state);
        }
    }

    fn reason_runtime(color: crate::ColorLevel) -> (Runtime<ReasonMenuApp>, Buffer) {
        let area = Rect::new(0, 0, 40, 8);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(
            ReasonMenuApp {
                state: MenuState::default(),
                chosen: None,
            },
            Theme::junie().downgrade(color),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        (runtime, buffer)
    }

    fn row_text(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| {
                buffer
                    .cell(Position::new(x, y))
                    .map_or(" ", ratatui_core::buffer::Cell::symbol)
            })
            .collect()
    }

    #[test]
    fn disabled_reason_is_bottom_border_help_across_color_profiles() {
        for color in [
            crate::ColorLevel::TrueColor,
            crate::ColorLevel::Ansi256,
            crate::ColorLevel::Ansi16,
            crate::ColorLevel::Mono,
        ] {
            let (mut runtime, mut buffer) = reason_runtime(color);
            assert_eq!(
                runtime.app().state.cursor(),
                0,
                "initial cursor at {color:?}"
            );
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Move,
                    pos: Position::new(4, 2),
                    mods: KeyModifiers::NONE,
                }),
            );
            assert_eq!(
                runtime.app().state.cursor(),
                0,
                "disabled hover at {color:?}"
            );
            assert_eq!(
                runtime.app().chosen,
                None,
                "disabled hover action at {color:?}"
            );
            runtime
                .draw_buffer(Rect::new(0, 0, 40, 8), &mut buffer)
                .commit_presented();
            assert!(
                row_text(&buffer, 7).contains(" The clipboard is empty "),
                "reason should paint in bottom border at {color:?}: {}",
                row_text(&buffer, 7)
            );
            let help = runtime
                .area_of_part(REASON_MENU, PartRef::of(Part::HELP))
                .expect("rendered status decoration");
            assert_eq!(help.y, 7);
            assert!(help.x >= 2 && help.right() <= 38);

            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Down,
                    pos: Position::new(4, 2),
                    mods: KeyModifiers::NONE,
                }),
            );
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Up,
                    pos: Position::new(4, 2),
                    mods: KeyModifiers::NONE,
                }),
            );
            assert_eq!(runtime.app().chosen, None, "disabled click at {color:?}");
        }
    }

    #[test]
    fn opted_in_keyboard_navigation_can_highlight_but_never_activate_disabled_reason() {
        let (mut runtime, mut buffer) = reason_runtime(crate::ColorLevel::TrueColor);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Down)));
        assert_eq!(runtime.app().state.cursor(), 1);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Enter)));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('p'))));
        assert_eq!(runtime.app().chosen, None);
        runtime
            .draw_buffer(Rect::new(0, 0, 40, 8), &mut buffer)
            .commit_presented();
        assert!(row_text(&buffer, 7).contains(" The clipboard is empty "));
    }

    #[test]
    fn keyboard_navigation_after_disabled_hover_suppresses_stale_pointer_reason() {
        let (mut runtime, mut buffer) = reason_runtime(crate::ColorLevel::TrueColor);
        let move_to = |runtime: &mut Runtime<ReasonMenuApp>, x, y| {
            let _ = crate::runtime::stub::deliver(
                runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Move,
                    pos: Position::new(x, y),
                    mods: KeyModifiers::NONE,
                }),
            );
        };
        move_to(&mut runtime, 4, 2);
        assert_eq!(runtime.app().state.pointed, Some(1));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Home)));
        assert_eq!(runtime.app().state.cursor(), 0);
        assert_eq!(runtime.app().state.pointed, None);
        assert!(runtime.app().state.hover_suppressed);
        runtime
            .draw_buffer(Rect::new(0, 0, 40, 8), &mut buffer)
            .commit_presented();
        assert!(!row_text(&buffer, 7).contains("The clipboard is empty"));

        move_to(&mut runtime, 4, 1);
        assert_eq!(runtime.app().state.cursor(), 0);
        assert!(!runtime.app().state.hover_suppressed);
        move_to(&mut runtime, 4, 3);
        assert_eq!(runtime.app().state.pointed, Some(2));
        runtime
            .draw_buffer(Rect::new(0, 0, 40, 8), &mut buffer)
            .commit_presented();
        assert!(row_text(&buffer, 7).contains("Nothing is selected"));
    }

    #[test]
    fn clear_on_leave_hides_highlight_until_keyboard_or_pointer_reenters() {
        let (mut runtime, _) = reason_runtime(crate::ColorLevel::Ansi256);
        let move_to = |runtime: &mut Runtime<ReasonMenuApp>, x, y| {
            let _ = crate::runtime::stub::deliver(
                runtime,
                Input::Mouse(Mouse {
                    kind: MouseKind::Move,
                    pos: Position::new(x, y),
                    mods: KeyModifiers::NONE,
                }),
            );
        };
        move_to(&mut runtime, 4, 1);
        move_to(&mut runtime, 0, 0);
        assert!(runtime.app().state.highlight_hidden);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Down)));
        assert!(!runtime.app().state.highlight_hidden);
        assert!(runtime.app().state.hover_suppressed);

        move_to(&mut runtime, 4, 1);
        assert!(!runtime.app().state.highlight_hidden);
        assert!(!runtime.app().state.hover_suppressed);
    }

    #[test]
    fn disabled_reason_is_a_noninteractive_part_with_style_and_slot_overrides() {
        const MENU: Id = Id::root("menu.reason.style");
        const ITEMS: [MenuItem<'static>; 1] =
            [MenuItem::new(REASON_PASTE, "Paste").reason("The clipboard is empty")];
        const CUSTOM: ratatui_core::style::Color = ratatui_core::style::Color::Rgb(21, 22, 23);
        const PATCH: [(Part, StylePatch); 1] = [(
            Part::HELP,
            StylePatch::new().set_fg(crate::theme::Role::Custom(CUSTOM)),
        )];
        let area = Rect::new(0, 0, 32, 5);
        let state = MenuState {
            pointed: Some(0),
            ..MenuState::default()
        };
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ContextMenu::at(MENU, &ITEMS, Position::new(0, 0))
                    .patch_part(&PATCH)
                    .draw(ui, area, &state);
            })
            .commit_presented();
        let help = runtime
            .area_of_part(MENU, PartRef::of(Part::HELP))
            .expect("help is decoration, not a hit target");
        assert_eq!(help.y, area.bottom() - 1);
        assert_eq!(
            buffer.cell(Position::new(help.x + 1, help.y)).unwrap().fg,
            CUSTOM
        );
        assert_ne!(
            runtime
                .projection_registry()
                .and_then(|registry| registry.hit(Position::new(help.x, help.y)))
                .map(|hit| hit.part),
            Some(PartRef::of(Part::HELP)),
            "help decoration is never registered as a pointer hit region"
        );

        let calls = std::cell::Cell::new(0);
        let painter = |ui: &mut Ui<'_>, slot: Rect| {
            calls.set(calls.get() + 1);
            ui.paint_str(slot, "?", ratatui_core::style::Style::default());
        };
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ContextMenu::at(MENU, &ITEMS, Position::new(0, 0))
                    .slot(Part::HELP, &painter)
                    .draw(ui, area, &state);
            })
            .commit_presented();
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn disabled_reason_handles_empty_enabled_override_unicode_and_narrow_areas() {
        const MENU: Id = Id::root("menu.reason.narrow");
        const EMPTY: [MenuItem<'static>; 0] = [];
        const UNICODE: [MenuItem<'static>; 1] =
            [MenuItem::new(REASON_PASTE, "Paste").reason("壊れた clipboard e\u{301}")];
        const ENABLED: [MenuItem<'static>; 1] = [MenuItem::new(REASON_PASTE, "Paste")
            .reason("This explanation is suppressed")
            .disabled(false)];

        for width in 0..=10 {
            let area = Rect::new(0, 0, width, 5);
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, area| {
                    ContextMenu::at(MENU, &UNICODE, Position::new(0, 0)).draw(
                        ui,
                        area,
                        &MenuState {
                            pointed: Some(0),
                            ..MenuState::default()
                        },
                    );
                })
                .commit_presented();
            let help = runtime.area_of_part(MENU, PartRef::of(Part::HELP));
            if width < 7 {
                assert_eq!(help, None, "no text fits in width {width}");
            } else {
                let help = help.expect("padded reason fits at width 7 and above");
                assert_eq!(help.y, area.bottom() - 1);
                assert!(help.x >= area.x + 2);
                assert!(help.right() <= area.right() - 2);
            }

            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, area| {
                    ContextMenu::at(MENU, &EMPTY, Position::new(0, 0)).draw(
                        ui,
                        area,
                        &MenuState::default(),
                    );
                })
                .commit_presented();
            assert_eq!(
                runtime.area_of_part(MENU, PartRef::of(Part::HELP)),
                None,
                "empty data has no reason at width {width}"
            );
        }

        let area = Rect::new(0, 0, 32, 5);
        let state = MenuState {
            pointed: Some(0),
            ..MenuState::default()
        };
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ContextMenu::at(MENU, &ENABLED, Position::new(0, 0)).draw(ui, area, &state);
            })
            .commit_presented();
        assert_eq!(runtime.area_of_part(MENU, PartRef::of(Part::HELP)), None);
        assert!(!row_text(&buffer, 4).contains("This explanation is suppressed"));
    }

    const ANCHOR_OWNER: Id = Id::root("menu.anchor.owner");
    const ANCHOR_MENU: Id = Id::root("menu.anchor.menu");
    const ANCHOR_MENUS: [Menu<'static>; 1] = [Menu::new("File", &ITEMS)];

    struct AnchorProbeApp {
        x: u16,
        render_owner: bool,
        request_open: bool,
        state: MenuState,
        anchor: Option<Anchor>,
    }

    impl App for AnchorProbeApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let menu = ContextMenu::at(ANCHOR_MENU, &ITEMS, Position::new(0, 0)).anchor_to_part(
                ANCHOR_OWNER,
                PartRef::item(Part::TITLE, ItemKey::index(0)),
                crate::layer::Side::Below,
                crate::layer::CrossAlign::Start,
            );
            let layer = menu.try_layer(cx);
            self.anchor = layer.map(|spec| spec.anchor);
            if self.request_open {
                self.request_open = false;
                if let Some(layer) = layer {
                    cx.open_layer(ANCHOR_MENU, layer);
                }
            }
            menu.update(cx, &mut self.state).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            if self.render_owner {
                let _ = MenuBar::new(ANCHOR_OWNER, &ANCHOR_MENUS).draw(
                    ui,
                    Rect::new(self.x, 4, 24, 1),
                    &MenuState::default(),
                );
            }
        }
    }

    #[test]
    fn component_part_anchor_tracks_live_bounds_and_fails_closed_when_absent() {
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(
            AnchorProbeApp {
                x: 5,
                render_owner: true,
                request_open: false,
                state: MenuState::default(),
                anchor: None,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let expected = runtime
            .area_of_part(ANCHOR_OWNER, PartRef::item(Part::TITLE, ItemKey::index(0)))
            .expect("the menu bar rendered the source title part");
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('z'))));
        assert_eq!(
            runtime.app().anchor,
            Some(Anchor::Rect {
                rect: expected,
                side: crate::layer::Side::Below,
                align: crate::layer::CrossAlign::Start,
            })
        );

        runtime.app_mut().x = 21;
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let moved = runtime
            .area_of_part(ANCHOR_OWNER, PartRef::item(Part::TITLE, ItemKey::index(0)))
            .expect("the source title moved and rendered again");
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('z'))));
        assert_eq!(
            runtime.app().anchor,
            Some(Anchor::Rect {
                rect: moved,
                side: crate::layer::Side::Below,
                align: crate::layer::CrossAlign::Start,
            })
        );

        runtime.app_mut().request_open = true;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('z'))));
        assert!(
            runtime.is_open(ANCHOR_MENU),
            "present owner opens anchored menu"
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        runtime.app_mut().x = 26;
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let moved_again = runtime
            .area_of_part(ANCHOR_OWNER, PartRef::item(Part::TITLE, ItemKey::index(0)))
            .expect("the source title moved while the menu stayed open");
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('z'))));
        assert_eq!(
            runtime.app().anchor,
            Some(Anchor::Rect {
                rect: moved_again,
                side: crate::layer::Side::Below,
                align: crate::layer::CrossAlign::Start,
            })
        );

        runtime.app_mut().render_owner = false;
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('z'))));
        assert_eq!(
            runtime.app().anchor,
            None,
            "missing owner part must not open a layer"
        );
        assert!(
            !runtime.is_open(ANCHOR_MENU),
            "missing owner closes the anchored layer"
        );
    }

    #[test]
    fn mono_pressed_menu_title_keeps_both_reserved_pads() {
        const ID: Id = Id::root("menu.tests.mono");
        const MENUS: [Menu<'static>; 1] = [Menu::new("File", &ITEMS)];
        let area = Rect::new(0, 0, 12, 1);
        let mut runtime = Runtime::new(
            crate::runtime::stub::Stub::default(),
            Theme::junie().downgrade(crate::ColorLevel::Mono),
        );
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.reference(
                    Some(
                        crate::ReferenceTarget::new(ID, crate::ReferenceState::PRESSED)
                            .part(PartRef::item(Part::TITLE, ItemKey::index(0))),
                    ),
                    |ui| MenuBar::new(ID, &MENUS).draw(ui, area, &MenuState::default()),
                );
            })
            .commit_presented();

        assert_eq!(
            buffer
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("["),
            "the title's left bracket occupies its leading pad"
        );
        assert_eq!(
            buffer
                .cell(Position::new(2, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("F"),
            "the title text starts inside the reserved pads"
        );
        assert_eq!(
            buffer
                .cell(Position::new(6, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("]"),
            "the title's right bracket occupies its trailing pad"
        );
    }

    #[test]
    fn submenu_is_typed_and_does_not_emit_parent_action() {
        let menu = ContextMenu::at(Id::root("menu.tests"), &ITEMS, Position::new(1, 1));
        let mut state = MenuState {
            cursor: 3,
            open: None,
            ..MenuState::default()
        };
        assert_eq!(
            menu.activate(&mut state),
            Some(MenuAction::Submenu(ItemKey::index(3)))
        );
    }

    #[test]
    fn move_changes_cursor_but_press_does_not() {
        let menu = ContextMenu::at(Id::root("menu.tests"), &ITEMS, Position::new(1, 1));
        let mut state = MenuState::default();
        let press = menu.pointer(&mut state, 2, Phase::Press);
        assert_eq!(state.cursor(), 0);
        assert!(press.is_consumed());
        let moved = menu.pointer(&mut state, 2, Phase::Move);
        assert_eq!(state.cursor(), 2);
        assert!(moved.is_consumed());
    }

    struct MenuApp {
        state: MenuState,
        chosen: Option<ActionKey>,
        keymap: KeyMap,
    }

    const BAR_MENUS: [Menu<'static>; 1] = [Menu::new("File", &ITEMS)];

    struct MenuBarApp {
        state: MenuState,
    }

    impl App for MenuBarApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            MenuBar::new(Id::root("menu.bar.runtime"), &BAR_MENUS)
                .update(cx, &mut self.state)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let _ = MenuBar::new(Id::root("menu.bar.runtime"), &BAR_MENUS).draw(
                ui,
                Rect::new(0, 0, 40, 10),
                &self.state,
            );
        }
    }

    const SHRINKING_MENUS: [Menu<'static>; 2] =
        [Menu::new("File", &ITEMS), Menu::new("Edit", &ITEMS)];

    struct ShrinkingMenuBarApp {
        state: MenuState,
        shrink: bool,
        closed: Option<DismissReason>,
    }

    impl App for ShrinkingMenuBarApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let menus = if self.shrink {
                &SHRINKING_MENUS[..1]
            } else {
                &SHRINKING_MENUS[..]
            };
            let response =
                MenuBar::new(Id::root("menu.bar.shrinking"), menus).update(cx, &mut self.state);
            if let Some(MenuAction::Closed(reason)) = response.action_ref() {
                self.closed = Some(*reason);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let menus = if self.shrink {
                &SHRINKING_MENUS[..1]
            } else {
                &SHRINKING_MENUS[..]
            };
            let _ = MenuBar::new(Id::root("menu.bar.shrinking"), menus).draw(
                ui,
                Rect::new(0, 0, 40, 10),
                &self.state,
            );
        }
    }

    impl App for MenuApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = ContextMenu::at(Id::root("menu.runtime"), &ITEMS, Position::new(0, 0))
                .update(cx, &mut self.state);
            if let Some(MenuAction::Chosen(action)) = response.action_ref() {
                self.chosen = Some(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ContextMenu::at(Id::root("menu.runtime"), &ITEMS, Position::new(0, 0)).draw(
                ui,
                Rect::new(0, 0, 30, 9),
                &self.state,
            );
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    const HINT_ACTION: ActionKey = ActionKey::custom("menu.hint.only");
    const CLICK_ACTION: ActionKey = ActionKey::custom("menu.hint.pointer-only");
    const HINT_ITEMS: [MenuItem<'static>; 2] = [
        MenuItem::new(HINT_ACTION, "Split right").hint("Ctrl+B %"),
        MenuItem::new(CLICK_ACTION, "Duplicate").hint("2×click"),
    ];

    struct HintMenuApp {
        state: MenuState,
        chosen: Option<ActionKey>,
    }

    impl App for HintMenuApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = ContextMenu::at(
                Id::root("menu.hint-only.runtime"),
                &HINT_ITEMS,
                Position::new(0, 0),
            )
            .update(cx, &mut self.state);
            if let Some(MenuAction::Chosen(action)) = response.action_ref() {
                self.chosen = Some(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let _ = ContextMenu::at(
                Id::root("menu.hint-only.runtime"),
                &HINT_ITEMS,
                Position::new(0, 0),
            )
            .draw(ui, Rect::new(0, 0, 40, 8), &self.state);
        }
    }

    fn menu_runtime() -> (Runtime<MenuApp>, Buffer) {
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(
            MenuApp {
                state: MenuState::default(),
                chosen: None,
                keymap: KeyMap::new(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        (runtime, buffer)
    }

    #[test]
    fn menu_bar_closes_live_layer_and_reports_close_when_open_menu_shrinks() {
        let area = Rect::new(0, 0, 40, 10);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(
            ShrinkingMenuBarApp {
                state: MenuState::default(),
                shrink: false,
                closed: None,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Right)));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Enter)));
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert_eq!(runtime.app().state.open, Some(1));
        assert!(runtime.is_open(Id::root("menu.bar.shrinking")));

        runtime.app_mut().shrink = true;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);

        assert_eq!(
            runtime.app().closed,
            Some(DismissReason::Programmatic),
            "shrinking away the open menu must emit its lifecycle close"
        );
        assert_eq!(runtime.app().state.open, None);
        assert!(!runtime.is_open(Id::root("menu.bar.shrinking")));
    }

    #[test]
    fn open_menu_bar_registers_only_the_dropdown_control() {
        let area = Rect::new(0, 0, 40, 10);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(
            MenuBarApp {
                state: MenuState::default(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Enter)));
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert_eq!(runtime.app().state.open, Some(0));
        assert!(
            !runtime.diagnostics().iter().any(|diagnostic| matches!(
                diagnostic,
                crate::diagnostics::Diagnostic::DuplicateId { id, .. }
                    if *id == Id::root("menu.bar.runtime")
            )),
            "the strip and dropdown both registered as the owner control"
        );
    }

    #[test]
    fn menu_bar_keyboard_and_pointer_activation_survive_all_color_levels() {
        let area = Rect::new(0, 0, 40, 10);
        for color in [
            crate::ColorLevel::TrueColor,
            crate::ColorLevel::Ansi256,
            crate::ColorLevel::Ansi16,
            crate::ColorLevel::Mono,
        ] {
            let mut buffer = Buffer::empty(area);
            let mut runtime = Runtime::new(
                MenuBarApp {
                    state: MenuState::default(),
                },
                Theme::junie().downgrade(color),
            );
            let _ = runtime.initialize();
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            runtime.draw_buffer(area, &mut buffer).commit_presented();

            let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Right)));
            let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Enter)));
            assert_eq!(runtime.app().state.open, Some(0), "keyboard at {color:?}");
            let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Esc)));
            assert_eq!(runtime.app().state.open, None, "escape at {color:?}");

            let mouse = |kind, x, y| {
                Input::Mouse(Mouse {
                    kind,
                    pos: Position::new(x, y),
                    mods: KeyModifiers::NONE,
                })
            };
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Move, 2, 0));
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 2, 0));
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            assert_eq!(
                buffer
                    .cell(Position::new(1, 0))
                    .map(ratatui_core::buffer::Cell::symbol),
                Some("["),
                "pressed left pad at {color:?}"
            );
            assert_eq!(
                buffer
                    .cell(Position::new(6, 0))
                    .map(ratatui_core::buffer::Cell::symbol),
                Some("]"),
                "pressed right pad at {color:?}"
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 0));
            assert_eq!(runtime.app().state.open, Some(0), "pointer at {color:?}");
        }
    }

    #[test]
    fn dynamic_item_binding_routes_remaps_removes_and_paints_effective_chord() {
        let owner = Id::root("menu.runtime");
        let (mut runtime, mut buffer) = menu_runtime();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('o'))));
        assert_eq!(runtime.app().chosen, Some(OPEN));
        runtime.app_mut().chosen = None;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('d'))));
        assert_eq!(runtime.app().chosen, None, "disabled item was published");

        runtime
            .app_mut()
            .keymap
            .remap_component(owner, OPEN, Chord::key(KeyCode::F(4)));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::Char('o'))));
        assert_eq!(runtime.app().chosen, None, "old raw key activated the item");
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::F(4))));
        assert_eq!(runtime.app().chosen, Some(OPEN));
        runtime
            .draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer)
            .commit_presented();
        let painted = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(painted.contains("F4"));
        assert!(!painted.contains(" o"));

        runtime.app_mut().chosen = None;
        runtime.app_mut().keymap.remove_component(owner, OPEN);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Key(key(KeyCode::F(4))));
        assert_eq!(runtime.app().chosen, None);
        runtime
            .draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer)
            .commit_presented();
        let painted = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(!painted.contains("F4"));
    }
}
