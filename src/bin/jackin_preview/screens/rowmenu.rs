//! A page-level row context menu: the library [`ContextMenu`] plus what it
//! is about (`T`, a row) and what each item runs (`C`, a command the page
//! already performs from a key or a button). Opened by a secondary click on
//! a row ([`ContextAnchor::Pointer`]) or by the context keys on the cursor
//! row ([`ContextAnchor::Control`]), per [`ContextConfig`].

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::context::{ContextAnchor, ContextConfig};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::menu::{ContextMenu, MenuEvent, MenuItem};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct Open<T, C> {
    target: T,
    menu: ContextMenu,
    commands: Vec<C>,
}

pub struct RowMenu<T, C> {
    /// Which keys open the menu at the cursor row, and whether a secondary
    /// click moves the cursor onto the row it opens for. Default
    /// [`ContextConfig::DEFAULT`] (Shift+F10 and the Menu key; moves).
    pub config: ContextConfig,
    id: WidgetId,
    open: Option<Open<T, C>>,
}

impl<T: Clone, C: Clone> RowMenu<T, C> {
    pub fn new(id: WidgetId) -> Self {
        Self {
            config: ContextConfig::DEFAULT,
            id,
            open: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// The page under the open menu draws without focus.
    pub fn hides_page_focus(&self) -> bool {
        self.open
            .as_ref()
            .is_some_and(|o| o.menu.hides_page_focus())
    }

    /// The open menu (its rectangle and rows as drawn last frame).
    #[cfg(test)]
    pub fn menu(&self) -> Option<&ContextMenu> {
        self.open.as_ref().map(|o| &o.menu)
    }

    /// Whether `key` asks for the menu of the cursor row.
    pub fn is_context_key(&self, key: &Key) -> bool {
        self.config.keys.matches(key)
    }

    /// Open the menu about `target` at `anchor`. Each item runs its
    /// command when chosen; an item with a reason never runs.
    pub fn open(
        &mut self,
        target: T,
        anchor: ContextAnchor,
        title: impl Into<String>,
        items: Vec<(MenuItem, C)>,
    ) {
        let (items, commands): (Vec<MenuItem>, Vec<C>) = items.into_iter().unzip();
        let menu = anchor.apply(ContextMenu::new(self.id, items).title(title));
        self.open = Some(Open {
            target,
            menu,
            commands,
        });
    }

    fn resolve(&mut self, ev: Option<MenuEvent>) -> Option<(T, C)> {
        match ev {
            Some(MenuEvent::Chosen(i)) => {
                let o = self.open.take()?;
                o.commands.get(i).cloned().map(|c| (o.target, c))
            }
            Some(MenuEvent::Dismissed) => {
                self.open = None;
                None
            }
            None => None,
        }
    }

    /// A key while open: the menu owns the keyboard. Returns the chosen
    /// command with its target.
    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<(T, C)>) {
        let Some(o) = self.open.as_mut() else {
            return (Outcome::Ignored, None);
        };
        let (out, ev) = o.menu.on_key(key);
        // the menu takes every key while it is open
        (out.or(Outcome::Consumed), self.resolve(ev))
    }

    /// A completed click while open: a row chooses, the backdrop dismisses
    /// (per the menu's `Dismiss`), the frame or a disabled row keeps it.
    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<(T, C)>) {
        let Some(o) = self.open.as_mut() else {
            return (Outcome::Ignored, None);
        };
        let ev = o.menu.on_click(id);
        (Outcome::Changed, self.resolve(ev))
    }

    /// Draw the open menu as an overlay. Call after the rows it may anchor
    /// to are drawn this frame.
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        if let Some(o) = self.open.as_mut() {
            o.menu.render(screen, buf, ctx);
        }
    }

    pub fn hints() -> Vec<Hint> {
        vec![
            hint("↑↓", "Move"),
            hint("Enter", "Choose"),
            hint("Esc", "Close"),
        ]
    }
}
