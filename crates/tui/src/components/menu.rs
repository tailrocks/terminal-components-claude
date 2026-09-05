//! Typed context menus and menu bars (`COMPONENT_ARCHITECTURE.md` §9.2,
//! §14.1, §18.2, Appendix A 4F).

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::keyhint::ChordText;
use super::{Acc, PartStyle, SlotFn, first_row, shift};
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{Anchor, Dismiss, DismissReason, LayerEvent, LayerSize, LayerSpec};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// One command row. Its optional [`Chord`] is both painted and handled; no
/// parallel display-only shortcut string exists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MenuItem<'a> {
    action: ActionKey,
    label: &'a str,
    chord: Option<Chord>,
    disabled: bool,
    danger: bool,
    separator_after: bool,
    submenu: Option<&'a [MenuItem<'a>]>,
}

impl<'a> MenuItem<'a> {
    /// An enabled command row.
    pub const fn new(action: ActionKey, label: &'a str) -> Self {
        MenuItem {
            action,
            label,
            chord: None,
            disabled: false,
            danger: false,
            separator_after: false,
            submenu: None,
        }
    }

    /// Attach the chord which both paints and activates this action.
    #[must_use]
    pub const fn chord(mut self, chord: Chord) -> Self {
        self.chord = Some(chord);
        self
    }

    /// Make the row inert but still visible.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// Use the destructive row variant.
    #[must_use]
    pub const fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Draw a separator after this row.
    #[must_use]
    pub const fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }

    /// Open `items` as a child menu instead of emitting this row's action.
    #[must_use]
    pub const fn submenu(mut self, items: &'a [MenuItem<'a>]) -> Self {
        self.submenu = Some(items);
        self
    }

    /// Typed action payload.
    pub const fn action(&self) -> ActionKey {
        self.action
    }

    /// Display label.
    pub const fn label(&self) -> &'a str {
        self.label
    }

    /// Shortcut used by both painting and update.
    pub const fn chord_ref(&self) -> Option<Chord> {
        self.chord
    }

    /// Whether activation is disabled.
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Child menu, if any.
    pub const fn submenu_ref(&self) -> Option<&'a [MenuItem<'a>]> {
        self.submenu
    }
}

/// One top-level menu in a [`MenuBar`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Menu<'a> {
    /// Label painted in the bar.
    pub label: &'a str,
    /// Rows shown by its dropdown.
    pub items: &'a [MenuItem<'a>],
}

impl<'a> Menu<'a> {
    /// A labelled menu.
    pub const fn new(label: &'a str, items: &'a [MenuItem<'a>]) -> Self {
        Menu { label, items }
    }
}

/// Menu state shared by a context menu and a menu bar dropdown.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MenuState {
    cursor: usize,
    open: Option<usize>,
}

impl MenuState {
    /// Current row index.
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Open top-level menu, for a [`MenuBar`].
    pub const fn open_menu(&self) -> Option<usize> {
        self.open
    }

    /// Whether a dropdown is open.
    pub const fn is_open(&self) -> bool {
        self.open.is_some()
    }
}

/// Semantic menu output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    /// A typed command was chosen.
    Chosen(ActionKey),
    /// A top-level dropdown opened.
    Opened(usize),
    /// A menu closed without choosing.
    Closed(DismissReason),
    /// A row requested its child menu.
    Submenu(ItemKey),
}

/// Const navigation commands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuCmd {
    /// Previous enabled row.
    Prev,
    /// Next enabled row.
    Next,
    /// First enabled row.
    First,
    /// Last enabled row.
    Last,
    /// Activate the cursor row.
    Activate,
    /// Previous top-level menu.
    PrevMenu,
    /// Next top-level menu.
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

/// Anchored popup content. The runtime owns placement, dismissal and z-order.
///
/// ## Construction
/// `ContextMenu::new(id, items, anchor)` or `ContextMenu::at(id, items,
/// position)`. Open [`ContextMenu::layer`] from `update`, then call `draw`
/// inside `ui.layer(id, ...)`.
///
/// ## Ownership
/// Caller owns borrowed [`MenuItem`]s and [`MenuState`]. Runtime owns focus,
/// hover, press and layer lifecycle.
///
/// ## Configuration
/// `.title`, `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::MENU`; dangerous items use `Variant::DANGER`.
///
/// ## States
/// Cursor row derives `ACTIVE`; rows also wear runtime `HOVERED` and item
/// `DISABLED`.
///
/// ## Actions
/// [`MenuAction::Chosen`] carries the row's [`ActionKey`]; submenu rows emit
/// [`MenuAction::Submenu`]; dismissal emits [`MenuAction::Closed`].
///
/// ## Focus
/// One focus stop for the popup; rows are keyed parts, not separate stops.
///
/// ## Keyboard
/// Arrows / `j` / `k`, Home, End, Enter and Space. An item's optional chord
/// is matched from the same field [`draw`](Self::draw) paints.
///
/// ## Mouse
/// Press selects an enabled row; click activates it. Disabled rows consume
/// without changing state.
///
/// ## Layout
/// Computes a [`LayerSize`], never a rect. Frame, optional title, command
/// rows and separators are clipped to the resolved layer area.
///
/// ## Parts
/// `CONTAINER`, `BORDER`, `TITLE`, `ROW`, `LABEL`, `KEY`, `MARKER`, `RULE`.
///
/// ## Overrides
/// `.patch` / `.patch_part` reach every part. Slots reach text and chrome;
/// `CONTAINER` and `ROW` remain owned surface fills.
///
/// ## Identity
/// Row regions use `PartRef::item(ROW, ItemKey::index(i))`; positional keys
/// are intentionally unstable because a menu declaration is static chrome.
///
/// ## Testing
/// `ContextMenuCase`; `render::components::context_menu::*`.
///
/// ## Invariants
/// Shortcut presentation and dispatch cannot drift. Disabled rows never
/// activate. Placement, flip and clamp exist only in the layer resolver.
pub struct ContextMenu<'a> {
    id: Id,
    items: &'a [MenuItem<'a>],
    anchor: Anchor,
    title: Option<&'a str>,
    ov: PartStyle<'a>,
}

impl fmt::Debug for ContextMenu<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextMenu")
            .field("id", &self.id)
            .field("items", &self.items.len())
            .field("anchor", &self.anchor)
            .field("title", &self.title)
            .finish_non_exhaustive()
    }
}

impl<'a> ContextMenu<'a> {
    /// Every styled part. `ROW` carries [`ItemKey::index`].
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::TITLE,
        Part::ROW,
        Part::LABEL,
        Part::KEY,
        Part::MARKER,
        Part::RULE,
    ];

    /// A menu anchored at `anchor`.
    pub const fn new(id: Id, items: &'a [MenuItem<'a>], anchor: Anchor) -> Self {
        ContextMenu {
            id,
            items,
            anchor,
            title: None,
            ov: PartStyle::new(),
        }
    }

    /// A context menu at a pointer position.
    pub const fn at(id: Id, items: &'a [MenuItem<'a>], position: Position) -> Self {
        Self::new(id, items, Anchor::Point(position))
    }

    /// Optional heading.
    #[must_use]
    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Instance patch.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(patches);
        self
    }

    /// Replace a paintable part.
    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    /// Size requested from the resolver. No rect is computed here.
    pub fn measured_size(&self, cx: &Cx<'_>) -> LayerSize {
        let d = cx.design();
        let width = self
            .natural_width(|action, chord| cx.effective_chord(self.id, action, chord))
            .clamp(d.size.popup_min_width, d.size.popup_max_width);
        LayerSize::Fixed(width, self.natural_height().max(3))
    }

    /// Popover specification. Outside click, focus-out and Esc dismiss it.
    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        LayerSpec::popover(self.id, self.anchor)
            .dismiss(Dismiss::ALL)
            .size(self.measured_size(cx))
    }

    fn natural_width(&self, effective: impl Fn(ActionKey, Option<Chord>) -> Option<Chord>) -> u16 {
        let title = self.title.map_or(0, width);
        let rows = self.items.iter().map(|item| {
            let shortcut = effective(item.action, item.chord).map_or(0, |chord| {
                width(ChordText::of(chord).as_str()).saturating_add(2)
            });
            width(item.label)
                .saturating_add(shortcut)
                .saturating_add(u16::from(item.submenu.is_some()))
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
            if self.enabled_at(index) {
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
            MenuCmd::First => set_cursor(st, self.first_enabled()),
            MenuCmd::Last => set_cursor(st, self.last_enabled()),
            MenuCmd::Activate => self
                .activate(st)
                .map_or_else(Response::consumed, Response::action),
            MenuCmd::PrevMenu | MenuCmd::NextMenu => Response::ignored(),
        }
    }

    fn pointer(&self, st: &mut MenuState, index: usize, phase: Phase) -> Response<MenuAction> {
        if !self.enabled_at(index) {
            return Response::consumed();
        }
        match phase {
            Phase::Click | Phase::DoubleClick => {
                st.cursor = index;
                self.activate(st)
                    .map_or_else(Response::consumed, Response::action)
            }
            Phase::Move if st.cursor != index => {
                st.cursor = index;
                Response::changed()
            }
            _ => Response::consumed(),
        }
    }

    /// Reconcile cursor, re-assert layer geometry, then process input.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut MenuState) -> Response<MenuAction> {
        if !self.enabled_at(st.cursor)
            && let Some(first) = self.first_enabled()
        {
            st.cursor = first;
        }
        if cx.is_open(self.id) {
            cx.resize_layer(self.id, self.measured_size(cx));
            cx.reanchor_layer(self.id, self.anchor);
        }
        let mut acc = Acc::new();
        for intent in cx.intents(self.id) {
            let response = match intent {
                Intent::Layer(LayerEvent::Dismissed(reason)) => {
                    Response::action(MenuAction::Closed(reason))
                }
                Intent::Layer(_) => Response::ignored().repaint(),
                Intent::Cancel => {
                    cx.close_layer(self.id, None);
                    Response::action(MenuAction::Closed(DismissReason::Esc))
                }
                Intent::Binding(action) => Binding::command(CONTEXT_BINDINGS, action).map_or_else(
                    || {
                        self.items
                            .iter()
                            .find(|item| !item.disabled && item.action == action)
                            .map_or_else(Response::ignored, |item| {
                                Response::action(MenuAction::Chosen(item.action))
                            })
                    },
                    |command| self.command(st, command),
                ),
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

    /// Paint menu content in the resolved layer `area`.
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
            ui.publish_bindings(self.id, live, CONTEXT_BINDINGS);
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
                let row = first_row(Rect { y, ..inner });
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
        });
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
        if FrameRead::hovered_part(ui, self.id)
            == Some(PartRef::item(Part::ROW, ItemKey::index(index)))
        {
            derived |= StateFlags::HOVERED;
        }
        if index == st.cursor {
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
        let key_width = effective_chord.map_or(0, |chord| width(ChordText::of(chord).as_str()));
        let label = Rect {
            width: label.width.saturating_sub(key_width.saturating_add(2)),
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
        if let Some(chord) = effective_chord {
            let text = ChordText::of(chord);
            let key = Rect {
                x: row.right().saturating_sub(key_width.saturating_add(1)),
                width: key_width.min(row.width),
                ..row
            };
            let key_style = self
                .ov
                .style(ui, self.id, Family::MENU, variant, Part::KEY, flags);
            paint_or_slot(ui, &self.ov, Part::KEY, key, text.as_str(), key_style.style);
        }
        ui.register_part(
            self.id,
            PartRef::item(Part::ROW, ItemKey::index(index)),
            row,
        );
    }

    /// Natural content size, bounded by `c`.
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

/// One-row menu strip whose open dropdown is layer content.
///
/// ## Construction
/// `MenuBar::new(id, menus)` over borrowed [`Menu`] declarations.
///
/// ## Ownership
/// Caller owns declarations and [`MenuState`]; runtime owns dropdown layer.
///
/// ## Configuration
/// `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::MENU`; dropdown danger rows use `Variant::DANGER`.
///
/// ## States
/// Cursor/open title derives `ACTIVE`; runtime supplies focus/hover/press.
///
/// ## Actions
/// `Opened(index)`, typed `Chosen(ActionKey)`, `Closed(reason)`.
///
/// ## Focus
/// Bar is one stop; open dropdown remains layer content owned by the bar.
///
/// ## Keyboard
/// Left/right (`h`/`l`) choose a menu; Down/Enter/Space opens it. Open-state
/// navigation uses [`ContextMenu`]'s table.
///
/// ## Mouse
/// Clicking a title opens its dropdown. Dropdown rows follow [`ContextMenu`].
///
/// ## Layout
/// One row; labels truncate as whole declarations when width runs out. Open
/// dropdown is painted with `ui.layer`, independent of call order.
///
/// ## Parts
/// Same typed part vocabulary as [`ContextMenu`].
///
/// ## Overrides
/// Forwarded to dropdown content so one instance override covers both.
///
/// ## Identity
/// Titles use positional `ItemKey`s; the dropdown layer uses bar `Id`.
///
/// ## Testing
/// `MenuBarCase`; `render::components::menu_bar::*`.
///
/// ## Invariants
/// Open dropdown size/anchor is re-asserted from update. Chords originate
/// only in [`MenuItem`], never a parallel label-string dispatcher.
pub struct MenuBar<'a> {
    id: Id,
    menus: &'a [Menu<'a>],
    ov: PartStyle<'a>,
}

impl fmt::Debug for MenuBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MenuBar")
            .field("id", &self.id)
            .field("menus", &self.menus.len())
            .finish_non_exhaustive()
    }
}

impl<'a> MenuBar<'a> {
    /// Bar plus dropdown parts.
    pub const PARTS: &'static [Part] = ContextMenu::PARTS;

    /// A bar over borrowed menus.
    pub const fn new(id: Id, menus: &'a [Menu<'a>]) -> Self {
        MenuBar {
            id,
            menus,
            ov: PartStyle::new(),
        }
    }

    /// Instance patch.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(patches);
        self
    }

    /// Replace a paintable part.
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
            title: None,
            ov: self.ov,
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

    /// Close a dropdown whose owning declaration is no longer present.
    ///
    /// `MenuBar` owns both the durable open index and the runtime layer.  A
    /// borrowed menu slice may change between phases, so clearing only the
    /// index would leave a live, unowned layer behind.  Treat that transition
    /// like any other programmatic dismissal and report it to the caller.
    fn close_missing(&self, cx: &mut Cx<'_>, st: &mut MenuState) -> Response<MenuAction> {
        let had_open_state = st.open.take().is_some();
        let had_live_layer = cx.is_open(self.id);
        if !had_open_state && !had_live_layer {
            return Response::ignored();
        }
        cx.close_layer(self.id, None);
        Response::action(MenuAction::Closed(DismissReason::Programmatic)).for_id(self.id)
    }

    /// Handle bar navigation and the active dropdown.
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
            if matches!(
                response.action_ref(),
                Some(MenuAction::Chosen(_) | MenuAction::Closed(_))
            ) {
                st.open = None;
                cx.close_layer(
                    self.id,
                    response.action_ref().and_then(|a| match a {
                        MenuAction::Chosen(key) => Some(*key),
                        _ => None,
                    }),
                );
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

    /// Paint the strip, then the open dropdown in its runtime layer.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &MenuState) -> Rect {
        let row = first_row(area);
        if row.is_empty() {
            return row;
        }
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
        ui.fill(row, container.style);
        if st.open.is_none() {
            ui.register_control(self.id, row, Focusability::Focusable);
        }
        ui.publish_bindings(self.id, live, BAR_BINDINGS);
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
            let style = self.ov.style(
                ui,
                self.id,
                Family::MENU,
                Variant::DEFAULT,
                Part::TITLE,
                flags,
            );
            let label = shift(rect, 1);
            if let Some(slot) = self.ov.slot_for(Part::TITLE) {
                slot(ui, label);
            } else if matches!(style.glyph, Slot::Set(GlyphRole::PressLeft)) {
                ui.glyph(Rect { width: 1, ..rect }, GlyphRole::PressLeft, style.style);
                ui.paint_str(label, menu.label, style.style);
                ui.glyph(
                    Rect {
                        x: rect.right().saturating_sub(1),
                        width: 1,
                        ..rect
                    },
                    GlyphRole::PressRight,
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
                title: None,
                ov: self.ov,
            };
            let _ = ui.layer(self.id, |ui, layer| dropdown.draw(ui, layer, st));
        }
        row
    }

    /// Natural strip width and one row.
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
    style: ratatui_core::style::Style,
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

    use super::*;
    use crate::event::{Input, Key, KeyModifiers};
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
    fn submenu_is_typed_and_does_not_emit_parent_action() {
        let menu = ContextMenu::at(Id::root("menu.tests"), &ITEMS, Position::new(1, 1));
        let mut state = MenuState {
            cursor: 3,
            open: None,
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
        runtime.draw_buffer(area, &mut buffer);
        runtime.draw_buffer(area, &mut buffer);
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
        runtime.draw_buffer(area, &mut buffer);
        runtime.draw_buffer(area, &mut buffer);
        let _ = runtime.handle(Input::Key(key(KeyCode::Right)));
        runtime.draw_buffer(area, &mut buffer);
        let _ = runtime.handle(Input::Key(key(KeyCode::Enter)));
        runtime.draw_buffer(area, &mut buffer);

        assert_eq!(runtime.app().state.open, Some(1));
        assert!(runtime.is_open(Id::root("menu.bar.shrinking")));

        runtime.app_mut().shrink = true;
        let _ = runtime.handle(Input::Tick);

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
        runtime.draw_buffer(area, &mut buffer);
        runtime.draw_buffer(area, &mut buffer);
        let _ = runtime.handle(Input::Key(key(KeyCode::Enter)));
        runtime.draw_buffer(area, &mut buffer);

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
    fn dynamic_item_binding_routes_remaps_removes_and_paints_effective_chord() {
        let owner = Id::root("menu.runtime");
        let (mut runtime, mut buffer) = menu_runtime();
        let _ = runtime.handle(Input::Key(key(KeyCode::Char('o'))));
        assert_eq!(runtime.app().chosen, Some(OPEN));
        runtime.app_mut().chosen = None;
        let _ = runtime.handle(Input::Key(key(KeyCode::Char('d'))));
        assert_eq!(runtime.app().chosen, None, "disabled item was published");

        runtime
            .app_mut()
            .keymap
            .remap_component(owner, OPEN, Chord::key(KeyCode::F(4)));
        let _ = runtime.handle(Input::Key(key(KeyCode::Char('o'))));
        assert_eq!(runtime.app().chosen, None, "old raw key activated the item");
        let _ = runtime.handle(Input::Key(key(KeyCode::F(4))));
        assert_eq!(runtime.app().chosen, Some(OPEN));
        runtime.draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer);
        let painted = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(painted.contains("F4"));
        assert!(!painted.contains(" o"));

        runtime.app_mut().chosen = None;
        runtime.app_mut().keymap.remove_component(owner, OPEN);
        let _ = runtime.handle(Input::Key(key(KeyCode::F(4))));
        assert_eq!(runtime.app().chosen, None);
        runtime.draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer);
        let painted = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(!painted.contains("F4"));
    }
}
