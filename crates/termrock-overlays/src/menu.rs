use core::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Color;

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

    #[must_use]
    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

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

    keep_underlay: bool,
    separator_inset: u16,
    skip_disabled: bool,
    clear_on_leave: bool,
    item_bindings: bool,
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
            .field("item_bindings", &self.item_bindings)
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
            keep_underlay: false,
            separator_inset: 0,
            skip_disabled: true,
            clear_on_leave: false,
            item_bindings: true,
        }
    }

    pub const fn at(id: Id, items: &'a [MenuItem<'a>], position: Position) -> Self {
        Self::new(id, items, Anchor::Point(position))
    }

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
    pub const fn keep_underlay(mut self) -> Self {
        self.keep_underlay = true;
        self
    }

    #[must_use]
    pub const fn separator_inset(mut self, cells: u16) -> Self {
        self.separator_inset = cells;
        self
    }

    #[must_use]
    pub const fn chord_case(mut self, case: ChordCase) -> Self {
        self.chord_case = case;
        self
    }

    #[must_use]
    pub const fn item_bindings(mut self, enabled: bool) -> Self {
        self.item_bindings = enabled;
        self
    }

    #[must_use]
    pub const fn skip_disabled(mut self, skip: bool) -> Self {
        self.skip_disabled = skip;
        self
    }

    #[must_use]
    pub const fn clear_on_leave(mut self, clear: bool) -> Self {
        self.clear_on_leave = clear;
        self
    }

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
            } else {
                let chord = if self.item_bindings {
                    effective(item.action, item.chord)
                } else {
                    item.chord
                };
                chord.map_or(label_width, |chord| {
                    label_width
                        .saturating_add(width(
                            ChordText::with_case(chord, self.chord_case).as_str(),
                        ))
                        .saturating_add(3)
                })
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

    fn item_reason(&self, index: usize) -> Option<&'a str> {
        let item = self.items.get(index)?;
        item.disabled
            .then_some(item.reason?)
            .filter(|reason| !reason.is_empty())
    }

    fn status_reason(&self, st: &MenuState) -> Option<&'a str> {
        st.pointed
            .and_then(|index| self.item_reason(index))
            .or_else(|| {
                (!st.highlight_hidden)
                    .then_some(st.cursor)
                    .and_then(|index| self.item_reason(index))
            })
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

    fn selection_at(&self, index: usize) -> Option<MenuSelection> {
        let item = self.items.get(index).filter(|item| !item.disabled)?;
        Some(if item.submenu.is_some() {
            (MenuAction::Submenu(ItemKey::index(index)), None)
        } else {
            (MenuAction::Chosen(item.action), Some(index))
        })
    }

    fn activate(&self, st: &mut MenuState) -> Option<MenuSelection> {
        self.selection_at(st.cursor)
    }

    fn command_with_selection(
        &self,
        st: &mut MenuState,
        command: MenuCmd,
    ) -> Response<MenuSelection> {
        match command {
            MenuCmd::Prev => no_selection(moved(self.step(st, -1))),
            MenuCmd::Next => no_selection(moved(self.step(st, 1))),
            MenuCmd::First => no_selection(set_cursor(st, self.navigation_cursor(true))),
            MenuCmd::Last => no_selection(set_cursor(st, self.navigation_cursor(false))),
            MenuCmd::Activate => self
                .activate(st)
                .map_or_else(Response::consumed, Response::action),
            MenuCmd::PrevMenu | MenuCmd::NextMenu => Response::ignored(),
        }
    }

    fn switch_menu(&self, delta: isize) -> Response<MenuSelection> {
        let Some((index, count)) = self.bar_navigation else {
            return Response::ignored();
        };
        let next = index
            .wrapping_add_signed(delta)
            .checked_rem(count)
            .unwrap_or_default();
        Response::action((MenuAction::Opened(next), None))
    }

    fn binding_response(&self, st: &mut MenuState, action: ActionKey) -> Response<MenuSelection> {
        if self.bar_navigation.is_some()
            && let Some(command) = Binding::command(BAR_BINDINGS, action)
        {
            return match command {
                MenuCmd::PrevMenu => self.switch_menu(-1),
                MenuCmd::NextMenu => self.switch_menu(1),
                command => self.command_with_selection(st, command),
            };
        }
        Binding::command(CONTEXT_BINDINGS, action).map_or_else(
            || {
                if self.item_bindings {
                    self.items
                        .iter()
                        .enumerate()
                        .find(|(_, item)| !item.disabled && item.action == action)
                        .map_or_else(Response::ignored, |(index, item)| {
                            Response::action((MenuAction::Chosen(item.action), Some(index)))
                        })
                } else {
                    Response::ignored()
                }
            },
            |command| self.command_with_selection(st, command),
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

    fn pointer(&self, st: &mut MenuState, index: usize, phase: Phase) -> Response<MenuSelection> {
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
                self.selection_at(index)
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
                no_selection(Response::changed())
            }
            _ => Response::consumed(),
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut MenuState) -> Response<MenuAction> {
        self.update_with_selection(cx, st).0
    }

    pub fn update_with_selection(
        &self,
        cx: &mut Cx<'_>,
        st: &mut MenuState,
    ) -> (Response<MenuAction>, Option<usize>) {
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
                return (
                    Response::action(MenuAction::Closed(DismissReason::Programmatic)),
                    None,
                );
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
                    Response::action((MenuAction::Closed(reason), None))
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
                    Response::action((MenuAction::Closed(DismissReason::Esc), None))
                }
                Intent::Binding(action) => {
                    if st.highlight_hidden && cx.activation_key().is_some() {
                        Response::consumed()
                    } else {
                        let navigation = self.is_navigation_binding(action);
                        let response = self.binding_response(st, action);
                        if navigation {
                            self.after_keyboard_navigation(st, cx);
                        }
                        response
                    }
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
            if let Some((action, selection)) = response.action_ref().copied() {
                if let MenuAction::Chosen(key) = action {
                    cx.close_layer(self.id, Some(key));
                }
                acc.action((action, selection));
            } else {
                acc.fold(&response.erase());
            }
        }
        finish_with_selection(acc, self.id)
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
            let container_bg = container.style.as_style().bg.unwrap_or(Color::Reset);
            if self.keep_underlay {
                ui.fill_keep_ink(area, container_bg);
            } else {
                ui.fill(area, container.style);
            }
            let border_style = self
                .ov
                .style(
                    ui,
                    self.id,
                    Family::MENU,
                    Variant::DEFAULT,
                    Part::BORDER,
                    live,
                )
                .style;
            let inner = ui.frame(area, border_style);
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
                    .filter(|item| self.item_bindings && !item.disabled)
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
                let style = self
                    .ov
                    .style(
                        ui,
                        self.id,
                        Family::MENU,
                        Variant::DEFAULT,
                        Part::TITLE,
                        live,
                    )
                    .style;
                paint_or_slot(ui, &self.ov, Part::TITLE, row, title, style);
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
                    let rule = self.separator_area(first_row(Rect { y, ..inner }));
                    let style = self.ov.style(
                        ui,
                        self.id,
                        Family::MENU,
                        Variant::DEFAULT,
                        Part::RULE,
                        live,
                    );
                    if !rule.is_empty() {
                        if let Some(slot) = self.ov.slot_for(Part::RULE) {
                            slot(ui, rule);
                        } else {
                            for cell in rule.rows() {
                                ui.glyph(cell, GlyphRole::RuleQuiet, style.style);
                            }
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
                | StateFlags::PRESSED
                | StateFlags::ACTIVE,
        );
        if !item.disabled
            && !st.hover_suppressed
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
        if pressed_row && !item.disabled {
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

        let mut row_paint = style.style;
        if row_paint.as_style().fg.is_none()
            && let Some(fg) = ui.theme_ref().color.fg.first().copied()
        {
            row_paint = row_paint.fg(fg);
        }
        ui.fill(row, row_paint);
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
        let effective_chord = if self.item_bindings {
            ui.effective_chord(self.id, item.action, item.chord)
        } else {
            item.chord
        };
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
        let mut effective_label_style = label_style.over(row_paint);
        let label_has_patch = self.ov.parts.iter().any(|(p, _)| *p == Part::LABEL);
        if !label_has_patch && let Some(fg) = row_paint.as_style().fg {
            effective_label_style = effective_label_style.fg(fg);
        }
        paint_or_slot(
            ui,
            &self.ov,
            Part::LABEL,
            label,
            item.label,
            effective_label_style,
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
            paint_or_slot(
                ui,
                &self.ov,
                Part::KEY,
                key,
                text,
                key_style.over(row_paint),
            );
        }
        ui.register_part(
            self.id,
            PartRef::item(Part::ROW, ItemKey::index(index)),
            row,
        );
    }

    fn separator_area(&self, rule: Rect) -> Rect {
        let inset = self.separator_inset.min(rule.width / 2);
        Rect {
            x: rule.x.saturating_add(inset),
            width: rule.width.saturating_sub(inset.saturating_mul(2)),
            ..rule
        }
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
            keep_underlay: false,
            separator_inset: 0,
            skip_disabled: true,
            clear_on_leave: false,
            item_bindings: true,
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
                    let variant = ui
                        .theme_ref()
                        .recipes
                        .get(Family::MENU)
                        .map_or(Variant::DEFAULT, |r| r.default_variant);
                    let target_variant = if variant == Variant::DEFAULT {
                        MENU_BAR_TITLE_VARIANT
                    } else {
                        Variant::DEFAULT
                    };
                    let local = self.ov.part_patch(Part::TITLE);
                    let style = ui.style_defaults(
                        Family::MENU,
                        target_variant,
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
            if let Some(slot) = self.ov.slot_for(Part::TITLE) {
                slot(ui, label);
            } else if matches!(style.glyph, Slot::Set(GlyphRole::PressLeft)) {
                ui.paint_str(label, menu.label, style.style);
                paint_pressed_bracket(
                    ui,
                    cell_at(rect, rect.x),
                    cell_at(rect, rect.right().saturating_sub(1)),
                    style.style,
                );
            } else {
                ui.paint_str(label, menu.label, style.style);
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
                keep_underlay: false,
                separator_inset: 0,
                skip_disabled: true,
                clear_on_leave: false,
                item_bindings: true,
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

type MenuSelection = (MenuAction, Option<usize>);

fn no_selection(response: Response<MenuAction>) -> Response<MenuSelection> {
    response.map_action(|action| (action, None))
}

fn finish_with_selection(acc: Acc<MenuSelection>, id: Id) -> (Response<MenuAction>, Option<usize>) {
    let response = acc.finish(id);
    let selection = match response.action_ref() {
        Some((MenuAction::Chosen(_), Some(index))) => Some(*index),
        _ => None,
    };
    (response.map_action(|(action, _)| action), selection)
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
