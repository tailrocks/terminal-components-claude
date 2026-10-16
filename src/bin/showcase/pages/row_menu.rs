//! A page's row context menu: the open menu, the row (or cell) it is
//! about, and the keys and click policy that open it. Every showcase page
//! with rows shares this plumbing, so each page only says which items a
//! row offers and what choosing one does.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use super::{Hint, PageEvent};
use junie_tui::core::event::{Key, Outcome};
use junie_tui::ui::context::ContextConfig;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::menu::{ContextMenu, MenuEvent};

/// The footer hints while a row menu is open.
pub const MENU_HINTS: [Hint; 3] = [("↑↓", "Move"), ("Enter", "Choose"), ("Esc", "Close")];

/// The footer hint advertising the keyboard context key.
pub const OPEN_HINT: Hint = ("Shift+F10", "Menu");

/// One open context menu about a `T` (a row, a `(row, col)` cell, ...).
pub struct RowMenu<T> {
    /// Which keys open the menu at the cursor and whether a secondary
    /// click moves the cursor. Default [`ContextConfig::DEFAULT`]
    /// (Shift+F10 and the Menu key; the click moves the cursor).
    pub config: ContextConfig,
    open: Option<(ContextMenu, T)>,
}

impl<T: Clone> Default for RowMenu<T> {
    fn default() -> Self {
        Self {
            config: ContextConfig::DEFAULT,
            open: None,
        }
    }
}

impl<T: Clone> RowMenu<T> {
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// While open the menu owns the keyboard: the page shows no focus.
    pub fn hides_page_focus(&self) -> bool {
        self.open
            .as_ref()
            .is_some_and(|(m, _)| m.hides_page_focus())
    }

    /// Open `menu` about `target`, replacing any open menu.
    pub fn open(&mut self, menu: ContextMenu, target: T) {
        self.open = Some((menu, target));
    }

    /// Draw the open menu (an overlay above the page).
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        if let Some((m, _)) = self.open.as_mut() {
            m.render(screen, buf, ctx);
        }
    }

    /// Route `ev` to the open menu. `None` when no menu is open (or the
    /// event is not the menu's); otherwise the outcome and, when an item
    /// was chosen, the target with the item's index. Esc and an outside
    /// click close the menu through its own dismiss policy.
    pub fn handle(&mut self, ev: &PageEvent) -> Option<(Outcome, Option<(T, usize)>)> {
        let (menu, _) = self.open.as_mut()?;
        let (out, event) = match ev {
            PageEvent::Key(key) => {
                let (o, e) = menu.on_key(key);
                // the menu owns the keyboard while it is open
                (o.or(Outcome::Consumed), e)
            }
            PageEvent::Click { id, .. } => (Outcome::Changed, menu.on_click(*id)),
            PageEvent::Press { .. } | PageEvent::Drag { .. } => (Outcome::Consumed, None),
            PageEvent::Secondary { id, .. } if menu.owns(*id) => (Outcome::Consumed, None),
            _ => return None,
        };
        match event {
            Some(MenuEvent::Chosen(i)) => {
                let (_, target) = self.open.take()?;
                Some((Outcome::Changed, Some((target, i))))
            }
            Some(MenuEvent::Dismissed) => {
                self.open = None;
                Some((Outcome::Changed, None))
            }
            None => Some((out, None)),
        }
    }
}

/// A plain key, for driving a widget through the same path its key
/// binding takes.
pub fn plain(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}
