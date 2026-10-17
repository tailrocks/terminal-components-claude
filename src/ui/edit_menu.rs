//! The standard Edit context menu every text control carries (M3).
//!
//! A secondary click on a text control's text (right click, or Ctrl+click
//! on macOS through
//! [`crate::core::event::InputConfig::ctrl_click_secondary`]), or the
//! keyboard context key while it holds focus ([`ContextKeys`]: Shift+F10
//! and the Menu key by default), opens one [`ContextMenu`] with the
//! desktop Edit commands:
//!
//! - an editable control: Cut, Copy, Paste, Select all;
//! - read-only text (a [`TextViewport`](crate::widgets::viewport::TextViewport),
//!   a [`ScrollPanel`](crate::widgets::panel::ScrollPanel), a read-only
//!   [`CodeEditor`](crate::widgets::code::CodeEditor)): Copy and Select
//!   all only.
//!
//! An unavailable command stays in the menu, dimmed, with its reason
//! ([`MenuItem::reason`]): Cut and Copy with nothing selected (or a
//! masked value: secure fields never hand their text out, as on macOS),
//! Paste while the field clipboard holds nothing (a terminal application
//! cannot read the system clipboard, so the terminal's own paste stays
//! the way to bring outside text in), Select all on empty text. Each
//! shortcut is the first key of the control's own key map the terminal
//! delivers ([`crate::core::keymap::label_for`]), so ⌘A shows only where
//! Cmd arrives.
//!
//! A chosen command runs through the control's own command path: an
//! [`EditField`] control through [`EditField::run`] (an
//! [`EditAction`](crate::widgets::field_common::EditAction) through
//! [`EditField::apply`], so Copy and Cut reach the field clipboard and its
//! OSC 52 hook, [`crate::core::edit::clipboard`]), a read-only viewer
//! through its Select All and Copy.
//!
//! The menu is owned by the control ([`EditMenuHost`]), so every page
//! and app that routes the control's events gets it without code of its
//! own: the control's [`MouseTarget::route`](crate::core::pointer::MouseTarget::route)
//! takes the menu's events, its `on_key` the menu's keys, and its render
//! queues the menu as an overlay above everything. While it is open the
//! control shows no focus ([`Interaction::focused_under`](crate::ui::ctx::Interaction::focused_under)),
//! so the menu holds the one focus indicator. [`EditMenu::enabled`] turns
//! it off for a control whose owner offers its own menu.
//!
//! Where a right click lands: inside the selection it keeps the selection
//! (the commands act on it); elsewhere in an editable control it places
//! the caret there first (Windows, GNOME and browser text fields), so
//! Paste lands where the user pointed. Read-only text keeps its
//! selection.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::core::edit::{TextCmd, clipboard};
use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::intent::Phase;
use crate::core::keymap::Actions;
use crate::core::pointer::{MouseTarget, PointerEvent};
use crate::ui::context::{ContextAnchor, ContextKeys};
use crate::ui::ctx::RenderCtx;
use crate::widgets::edit::{EditField, Editable};
use crate::widgets::menu::{ContextMenu, MenuAction, MenuItem};

/// What the Edit menu can offer for a control right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditCaps {
    /// The text can change: Cut and Paste are offered.
    pub editable: bool,
    /// Something is selected: Cut and Copy are available.
    pub selection: bool,
    /// There is text to select: Select all is available.
    pub text: bool,
    /// The value is masked (a password): Cut and Copy never hand it out.
    pub secret: bool,
}

impl EditCaps {
    /// The caps of an [`EditField`]: editable unless read-only.
    pub fn of(field: &EditField) -> Self {
        Self {
            editable: !field.read_only,
            selection: field.buffer.selected_text().is_some(),
            text: !field.buffer.text().is_empty(),
            secret: false,
        }
    }

    /// Read-only text: Copy and Select all only.
    pub fn read_only(selection: bool, text: bool) -> Self {
        Self {
            editable: false,
            selection,
            text,
            secret: false,
        }
    }

    /// See [`Self::secret`].
    pub fn secret(mut self, on: bool) -> Self {
        self.secret = on;
        self
    }
}

/// The commands and rows of an Edit menu for `caps`, with each command's
/// shortcut from `shortcut` (the first deliverable key of the control's
/// key map, `None` to show none).
pub fn edit_items(
    caps: EditCaps,
    shortcut: impl Fn(TextCmd) -> Option<String>,
) -> Vec<(TextCmd, MenuItem)> {
    let mut cmds = Vec::with_capacity(4);
    if caps.editable {
        cmds.push(TextCmd::Cut);
    }
    cmds.push(TextCmd::Copy);
    if caps.editable {
        cmds.push(TextCmd::Paste);
    }
    cmds.push(TextCmd::SelectAll);
    let pasteable = clipboard::text().is_some_and(|t| !t.is_empty());
    cmds.into_iter()
        .map(|cmd| {
            let reason = match cmd {
                TextCmd::Cut | TextCmd::Copy if caps.secret => Some("Hidden text is not copied"),
                TextCmd::Cut | TextCmd::Copy if !caps.selection => Some("Nothing is selected"),
                TextCmd::Paste if !pasteable => Some("Nothing copied yet"),
                TextCmd::SelectAll if !caps.text => Some("No text to select"),
                _ => None,
            };
            let mut item = MenuItem::new(cmd.label());
            if let Some(s) = shortcut(cmd) {
                item = item.shortcut(s);
            }
            if let Some(r) = reason {
                item = item.reason(r);
            }
            // Select all stands apart from the clipboard commands
            let item = item
                .separator_if(cmd == TextCmd::Paste || (!caps.editable && cmd == TextCmd::Copy));
            (cmd, item)
        })
        .collect()
}

/// The Edit context menu a text control owns: whether it opens, which
/// keys open it, and the open menu.
#[derive(Debug, Clone)]
pub struct EditMenu {
    /// A secondary click on the text or a context key opens the menu.
    /// Default `true`; `false` leaves the secondary click and the keys to
    /// the owner (an app with its own menu for the control).
    pub enabled: bool,
    /// The keys that open the menu at the caret while the control holds
    /// focus. Default [`ContextKeys::DEFAULT`] (Shift+F10 and the Menu
    /// key).
    pub keys: ContextKeys,
    /// Boxed: a closed menu costs every text control one pointer.
    open: Option<Box<(ContextMenu, Vec<TextCmd>)>>,
    /// A command chosen from the menu changed the text since the last
    /// [`Self::take_edited`].
    edited: bool,
    /// The control's area and caret as last drawn: where a keyboard-opened
    /// menu goes.
    area: Rect,
    caret: Option<Position>,
}

impl Default for EditMenu {
    fn default() -> Self {
        Self {
            enabled: true,
            keys: ContextKeys::DEFAULT,
            open: None,
            edited: false,
            area: Rect::ZERO,
            caret: None,
        }
    }
}

impl EditMenu {
    /// The menu's id for the control `owner`.
    pub fn id_for(owner: WidgetId) -> WidgetId {
        owner.sub("edit-menu")
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// The open menu, if any.
    pub fn menu(&self) -> Option<&ContextMenu> {
        self.open.as_ref().map(|o| &o.0)
    }

    /// Close the menu without running a command.
    pub fn close(&mut self) -> Outcome {
        if self.open.take().is_some() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    /// Whether a command chosen from the menu changed the text since the
    /// last call, clearing the flag. A control's `on_key` reports it as
    /// its own change action; an owner routing pointer events (whose
    /// response carries no action) reads it after a menu click.
    pub fn take_edited(&mut self) -> bool {
        std::mem::take(&mut self.edited)
    }

    /// Whether `ev` belongs to the open menu (its rows, its backdrop).
    pub fn owns(&self, ev: &PointerEvent) -> bool {
        self.open.as_ref().is_some_and(|o| ev.on(o.0.id))
    }

    /// Open the menu for `owner` with `items` at `anchor`.
    pub fn open(
        &mut self,
        owner: WidgetId,
        anchor: ContextAnchor,
        items: Vec<(TextCmd, MenuItem)>,
    ) {
        let (cmds, rows): (Vec<_>, Vec<_>) = items.into_iter().unzip();
        let menu = anchor.apply(ContextMenu::new(Self::id_for(owner), rows));
        self.open = Some(Box::new((menu, cmds)));
    }

    /// Where a keyboard-opened menu goes: under the caret as last drawn,
    /// else under the control's first row.
    pub fn key_anchor(&self, owner: WidgetId) -> ContextAnchor {
        match self.caret {
            Some(c) => ContextAnchor::Pointer(c),
            None => ContextAnchor::Control {
                id: owner,
                fallback: Rect::new(self.area.x, self.area.y, self.area.width.max(1), 1),
            },
        }
    }

    /// An open menu's answer to `key`: `None` when no menu is open, or a
    /// key the menu has no use for closed it (the control takes the key,
    /// K14); otherwise the outcome and the chosen command.
    pub fn on_key(&mut self, key: &Key) -> Option<(Outcome, Option<TextCmd>)> {
        let menu = &mut self.open.as_mut()?.0;
        let (out, action) = menu.on_key(key);
        match action {
            Some(MenuAction::Dismissed) if out == Outcome::Ignored => {
                self.open = None;
                None
            }
            other => Some(self.settle(out.or(Outcome::Consumed), other)),
        }
    }

    /// An open menu's answer to one of its pointer events (`None` for any
    /// other event).
    pub fn on_pointer(&mut self, ev: &PointerEvent) -> Option<(Outcome, Option<TextCmd>)> {
        if !self.owns(ev) {
            return None;
        }
        let menu = &mut self.open.as_mut()?.0;
        let mut r = menu.pointer(ev);
        let action = r.take_action();
        Some(self.settle(r.outcome(), action))
    }

    fn settle(&mut self, out: Outcome, action: Option<MenuAction>) -> (Outcome, Option<TextCmd>) {
        match action {
            Some(MenuAction::Chosen(i)) => {
                let cmd = self.open.take().and_then(|o| o.1.get(i).copied());
                (Outcome::Changed, cmd)
            }
            Some(MenuAction::Dismissed) => {
                self.open = None;
                (Outcome::Changed, None)
            }
            None => (out, None),
        }
    }

    /// Queue the open menu above everything (call at the end of the
    /// control's render, with the control's `area`). While it is open the
    /// terminal caret is hidden: the menu holds the keyboard.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.area = area;
        self.caret = ctx.cursor.filter(|c| area.contains(*c));
        if let Some(o) = self.open.as_mut() {
            let menu = &mut o.0;
            ctx.cursor = None;
            let screen = *buf.area();
            menu.render(screen, buf, ctx);
        }
    }
}

/// A control that owns an [`EditMenu`]. Implementing it is all a text
/// control does: [`edit_menu_key`], [`edit_menu_pointer`] and
/// [`EditMenu::render`] are the one behaviour every text control shares.
pub trait EditMenuHost {
    /// The control's menu.
    fn edit_menu(&mut self) -> &mut EditMenu;
    /// The id whose secondary click opens the menu.
    fn edit_owner(&self) -> WidgetId;
    /// What the menu can offer now.
    fn edit_caps(&self) -> EditCaps;
    /// The shortcut `cmd` shows: the first deliverable key the control
    /// binds to it.
    fn edit_shortcut(&self, cmd: TextCmd) -> Option<String>;
    /// Whether a secondary click on the control at `ev` opens the menu
    /// (the text, not a label or a scrollbar). Default: anywhere on it.
    fn edit_target(&self, _ev: &PointerEvent) -> bool {
        true
    }
    /// A secondary click at `pos` opens the menu: place the caret there
    /// unless `pos` is inside the selection. Default: nothing moves.
    fn edit_point(&mut self, _pos: Position) {}
    /// Run a chosen command; `true` when it changed the text.
    fn edit_run(&mut self, cmd: TextCmd) -> bool;
}

fn run<H: EditMenuHost + ?Sized>(h: &mut H, out: Outcome, cmd: Option<TextCmd>) -> Outcome {
    let Some(c) = cmd else {
        return out;
    };
    if h.edit_run(c) {
        h.edit_menu().edited = true;
    }
    Outcome::Changed
}

fn open_for<H: EditMenuHost + ?Sized>(h: &mut H, anchor: ContextAnchor) {
    let owner = h.edit_owner();
    let items = edit_items(h.edit_caps(), |c| h.edit_shortcut(c));
    h.edit_menu().open(owner, anchor, items);
}

/// The Edit menu's part of a control's `on_key`: the open menu takes the
/// key (a chosen command runs through [`EditMenuHost::edit_run`]); a
/// context key opens it at the caret. `None`: the control handles `key`.
pub fn edit_menu_key<H: EditMenuHost + ?Sized>(h: &mut H, key: &Key) -> Option<Outcome> {
    if let Some((out, cmd)) = h.edit_menu().on_key(key) {
        return Some(run(h, out, cmd));
    }
    let owner = h.edit_owner();
    let menu = h.edit_menu();
    if !menu.enabled || !menu.keys.matches(key) {
        return None;
    }
    let anchor = menu.key_anchor(owner);
    open_for(h, anchor);
    Some(Outcome::Changed)
}

/// The Edit menu's part of a control's pointer handling: the open menu's
/// rows and backdrop, and a secondary click on the control, which opens
/// it at the pointer. `None`: the control handles `ev`.
pub fn edit_menu_pointer<H: EditMenuHost + ?Sized>(
    h: &mut H,
    ev: &PointerEvent,
) -> Option<Outcome> {
    if let Some((out, cmd)) = h.edit_menu().on_pointer(ev) {
        return Some(run(h, out, cmd));
    }
    if !ev.is(Phase::Secondary)
        || !ev.on(h.edit_owner())
        || !h.edit_menu().enabled
        || !h.edit_target(ev)
    {
        return None;
    }
    h.edit_point(ev.pos());
    open_for(h, ContextAnchor::Pointer(ev.pos()));
    Some(Outcome::Changed)
}

/// [`MouseTarget::route`] for a control with an Edit menu: its own events
/// and its open menu's.
pub fn edit_menu_route<H>(
    h: &mut H,
    ev: &PointerEvent,
) -> Option<crate::core::response::Response<H::Action>>
where
    H: EditMenuHost + MouseTarget + ?Sized,
{
    let mine = ev.on(h.mouse_id());
    let menu = h.edit_menu().owns(ev);
    (mine || menu).then(|| h.pointer(ev))
}

/// [`EditMenuHost::edit_point`] for an [`Editable`] control: the pointer
/// owns this focus (the keyboard's focus selection does not apply), the
/// edit starts or continues as a press would start it, and the caret goes
/// where the user pointed unless that is inside the selection.
pub fn point_editable<T: Editable + ?Sized>(t: &mut T, pos: Position) {
    t.pointer_focus();
    t.pointer_begin();
    let at = t.offset_at(pos);
    point_field(t.field(), at);
    t.pointer_after();
}

/// [`EditMenuHost::edit_target`] for an [`Editable`] control: its text,
/// not its label, while the pointer acts on it.
pub fn editable_target<T: Editable + ?Sized>(t: &T, ev: &PointerEvent) -> bool {
    t.pointer_enabled() && !t.on_label(ev.pos())
}

/// Place the caret of `field` at `offset` for a secondary click, unless
/// the offset lies inside the selection (which the commands then act on).
pub fn point_field(field: &mut EditField, offset: usize) {
    if field
        .buffer
        .selection()
        .is_some_and(|r| r.start <= offset && offset <= r.end)
    {
        return;
    }
    field.buffer.select_range(offset, offset);
    field.buffer.clear_selection();
}

#[cfg(test)]
#[path = "edit_menu_tests.rs"]
mod tests;
