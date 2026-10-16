//! Universal context menus: one hook every row widget shares.
//!
//! A secondary click (right click, or Ctrl+click on macOS through
//! [`crate::core::event::InputConfig::ctrl_click_secondary`]) on a row, a
//! cell or a tab, or the keyboard context key ([`ContextKeys`]: Shift+F10
//! and the Menu key by default) on the widget's cursor, turns into a
//! [`ContextRequest`]: *what* the menu is about ([`ContextSource::Target`])
//! and *where* it opens ([`ContextAnchor`]). The owner answers with the
//! items it can offer for that target and opens a
//! [`ContextMenu`](crate::widgets::menu::ContextMenu) through
//! [`ContextRequest::menu`]; unavailable commands stay in the menu with
//! [`MenuItem::reason`](crate::widgets::menu::MenuItem::reason).
//!
//! The router side (a right-click outside an open popup closes it and
//! re-opens at the new spot) is
//! [`PointerState::route_secondary`](crate::core::pointer::PointerState::route_secondary).

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::core::event::Key;
use crate::core::id::WidgetId;
use crate::widgets::grid::DataGrid;
use crate::widgets::list::ListBox;
use crate::widgets::menu::{ContextMenu, MenuItem};
use crate::widgets::table::DataTable;
use crate::widgets::tabs::Tabs;
use crate::widgets::tree::TreeView;

/// Which keys open the context menu of the focused widget's cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextKeys {
    /// Shift+F10 (the desktop keyboard context-menu chord). Default on.
    pub shift_f10: bool,
    /// The Menu (application) key, where the terminal reports it. Default
    /// on.
    pub menu_key: bool,
    /// One more key an app may add (e.g. a plain letter on a page with no
    /// text entry). Default `None`.
    pub extra: Option<Key>,
}

impl ContextKeys {
    /// Shift+F10 and the Menu key.
    pub const DEFAULT: Self = Self {
        shift_f10: true,
        menu_key: true,
        extra: None,
    };
    /// No key opens a context menu.
    pub const NONE: Self = Self {
        shift_f10: false,
        menu_key: false,
        extra: None,
    };

    pub const fn shift_f10(mut self, on: bool) -> Self {
        self.shift_f10 = on;
        self
    }
    pub const fn menu_key(mut self, on: bool) -> Self {
        self.menu_key = on;
        self
    }
    pub const fn extra(mut self, key: Option<Key>) -> Self {
        self.extra = key;
        self
    }

    /// Whether `key` asks for the context menu.
    pub fn matches(&self, key: &Key) -> bool {
        (self.shift_f10 && key.code == KeyCode::F(10) && key.mods == KeyModifiers::SHIFT)
            || (self.menu_key && key.code == KeyCode::Menu && key.plain())
            || self
                .extra
                .is_some_and(|k| k.code == key.code && k.mods == key.mods)
    }
}

impl Default for ContextKeys {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Context-menu behaviour shared by every row widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextConfig {
    /// Keys that open the menu at the cursor.
    pub keys: ContextKeys,
    /// A secondary click moves the widget's cursor onto the row, cell or
    /// tab it opens the menu for (Finder, Mail), so the menu's commands
    /// visibly apply to the highlighted item. Default `true`.
    pub moves_cursor: bool,
}

impl ContextConfig {
    pub const DEFAULT: Self = Self {
        keys: ContextKeys::DEFAULT,
        moves_cursor: true,
    };

    pub const fn keys(mut self, keys: ContextKeys) -> Self {
        self.keys = keys;
        self
    }
    pub const fn moves_cursor(mut self, on: bool) -> Self {
        self.moves_cursor = on;
        self
    }

    /// A secondary click on hit `id` at `pos`: the request when `id` is
    /// one of `src`'s rows, cells or tabs (moving the cursor there per
    /// [`Self::moves_cursor`]).
    pub fn on_secondary<S: ContextSource + ?Sized>(
        &self,
        src: &mut S,
        id: WidgetId,
        pos: Position,
    ) -> Option<ContextRequest<S::Target>> {
        let target = src.context_target(id)?;
        if self.moves_cursor {
            src.context_select(target);
        }
        Some(ContextRequest {
            target,
            anchor: ContextAnchor::Pointer(pos),
        })
    }

    /// A key: the request for the cursor when `key` is a context key.
    pub fn on_key<S: ContextSource + ?Sized>(
        &self,
        src: &S,
        key: &Key,
    ) -> Option<ContextRequest<S::Target>> {
        if !self.keys.matches(key) {
            return None;
        }
        src.context_cursor()
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Where a context menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAnchor {
    /// At the pointer (a secondary click).
    Pointer(Position),
    /// Below a control as drawn this frame (the cursor row, cell or tab),
    /// with `fallback` used when it is not drawn before the menu.
    Control { id: WidgetId, fallback: Rect },
}

impl ContextAnchor {
    /// Anchor `menu` here.
    pub fn apply(self, menu: ContextMenu) -> ContextMenu {
        match self {
            Self::Pointer(pos) => menu.at(pos),
            Self::Control { id, fallback } => menu.anchor_control(id, fallback),
        }
    }
}

/// A context menu asked for `target`, to open at `anchor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextRequest<T> {
    pub target: T,
    pub anchor: ContextAnchor,
}

impl<T> ContextRequest<T> {
    /// The menu answering this request.
    pub fn menu(&self, id: WidgetId, items: Vec<MenuItem>) -> ContextMenu {
        self.anchor.apply(ContextMenu::new(id, items))
    }

    /// The same request about another target (an owner mapping a display
    /// row to its own record).
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ContextRequest<U> {
        ContextRequest {
            target: f(self.target),
            anchor: self.anchor,
        }
    }
}

/// A widget whose rows, cells or tabs carry a context menu.
pub trait ContextSource {
    /// What a menu is about: a row index, a `(row, col)` cell, a tab.
    type Target: Copy;
    /// The target a hit id stands for, if it is one of the widget's items.
    fn context_target(&self, id: WidgetId) -> Option<Self::Target>;
    /// Move the cursor onto `target`.
    fn context_select(&mut self, target: Self::Target);
    /// The cursor's target and where its menu opens, `None` when empty.
    fn context_cursor(&self) -> Option<ContextRequest<Self::Target>>;
}

/// A one-row rectangle at `row` rows below the top of `area`'s visible
/// part (the anchor used before a row is drawn).
fn row_rect(area: Rect, row: usize, offset: usize) -> Rect {
    let dy = row
        .saturating_sub(offset)
        .min(area.height.saturating_sub(1) as usize) as u16;
    Rect::new(area.x, area.y + dy, area.width.max(1), 1)
}

impl ContextSource for ListBox {
    type Target = usize;
    fn context_target(&self, id: WidgetId) -> Option<usize> {
        self.locate(id)
    }
    fn context_select(&mut self, row: usize) {
        if row < self.items.len() {
            self.cursor = row;
        }
    }
    fn context_cursor(&self) -> Option<ContextRequest<usize>> {
        (self.cursor < self.items.len()).then(|| ContextRequest {
            target: self.cursor,
            anchor: ContextAnchor::Control {
                id: self.row_id(self.cursor),
                fallback: row_rect(self.area, self.cursor, self.scroll.offset),
            },
        })
    }
}

impl ContextSource for TreeView {
    type Target = usize;
    fn context_target(&self, id: WidgetId) -> Option<usize> {
        self.locate(id).map(|(i, _)| i)
    }
    fn context_select(&mut self, row: usize) {
        self.cursor = row;
    }
    fn context_cursor(&self) -> Option<ContextRequest<usize>> {
        self.cursor_path()?;
        Some(ContextRequest {
            target: self.cursor,
            anchor: ContextAnchor::Control {
                id: self.row_id(self.cursor),
                fallback: row_rect(self.area, self.cursor, self.scroll.offset),
            },
        })
    }
}

impl ContextSource for DataTable {
    /// `(display row, column)`; a click on a row's free space names the
    /// cursor column.
    type Target = (usize, usize);
    fn context_target(&self, id: WidgetId) -> Option<(usize, usize)> {
        self.locate(id)
            .map(|(r, c)| (r, c.unwrap_or(self.cursor_col)))
    }
    fn context_select(&mut self, (row, col): (usize, usize)) {
        if row < self.rows.len() {
            self.cursor_row = row;
            if self.cell_nav && col < self.columns.len() {
                self.cursor_col = col;
            }
        }
    }
    fn context_cursor(&self) -> Option<ContextRequest<(usize, usize)>> {
        (self.cursor_row < self.rows.len()).then(|| {
            let id = if self.cell_nav {
                self.cell_id(self.cursor_row, self.cursor_col)
            } else {
                self.row_id(self.cursor_row)
            };
            ContextRequest {
                target: (self.cursor_row, self.cursor_col),
                anchor: ContextAnchor::Control {
                    id,
                    // header row above the body
                    fallback: row_rect(
                        Rect {
                            y: self.area.y.saturating_add(1),
                            height: self.area.height.saturating_sub(1),
                            ..self.area
                        },
                        self.cursor_row,
                        self.scroll.offset,
                    ),
                },
            }
        })
    }
}

impl ContextSource for DataGrid {
    /// `(display row, column)`; the row-number gutter names the cursor
    /// column.
    type Target = (usize, usize);
    fn context_target(&self, id: WidgetId) -> Option<(usize, usize)> {
        self.locate(id)
            .or_else(|| self.locate_rownum(id).map(|r| (r, self.cursor.1)))
    }
    fn context_select(&mut self, cell: (usize, usize)) {
        if cell.0 < self.len() && cell.1 < self.columns.len() {
            self.cursor = cell;
        }
    }
    fn context_cursor(&self) -> Option<ContextRequest<(usize, usize)>> {
        (self.cursor.0 < self.len()).then(|| ContextRequest {
            target: self.cursor,
            anchor: ContextAnchor::Control {
                id: self.cell_id(self.cursor.0, self.cursor.1),
                fallback: row_rect(
                    Rect {
                        y: self.area.y.saturating_add(1),
                        height: self.area.height.saturating_sub(1),
                        ..self.area
                    },
                    self.cursor.0,
                    self.scroll.offset,
                ),
            },
        })
    }
}

impl ContextSource for Tabs {
    type Target = usize;
    fn context_target(&self, id: WidgetId) -> Option<usize> {
        self.locate(id)
    }
    fn context_select(&mut self, tab: usize) {
        if tab < self.items.len() {
            self.cursor = tab;
        }
    }
    fn context_cursor(&self) -> Option<ContextRequest<usize>> {
        (self.cursor < self.items.len()).then(|| ContextRequest {
            target: self.cursor,
            anchor: ContextAnchor::Control {
                id: self.tab_id(self.cursor),
                fallback: self.areas.get(self.cursor).copied().unwrap_or_default(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::core::pointer::{PointerConfig, PointerState, SecondaryOutside, SecondaryRoute};
    use crate::theme::Theme;
    use crate::ui::ctx::{Interaction, RenderCtx};
    use crate::widgets::list::{ListItem, SelectMode};
    use crate::widgets::menu::MenuAction;
    use ratatui::buffer::Buffer;

    const MENU: WidgetId = WidgetId::of("ctx.menu");

    fn list() -> ListBox {
        let items = (0..8).map(|i| ListItem::new(&format!("Row {i}"))).collect();
        ListBox::new(WidgetId::of("ctx.list"), items, SelectMode::Single)
    }

    fn items() -> Vec<MenuItem> {
        vec![MenuItem::new("Open"), MenuItem::new("Delete").danger()]
    }

    /// Draw the list, then `menu` above it; the hit registry of the frame.
    fn frame(l: &mut ListBox, menu: Option<&mut ContextMenu>) -> HitRegistry {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let screen = Rect::new(0, 0, 40, 14);
        let mut buf = Buffer::empty(screen);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            l.render(Rect::new(0, 2, 40, 10), &mut buf, &mut ctx, theme.canvas);
            if let Some(m) = menu {
                m.render(screen, &mut buf, &mut ctx);
            }
            ctx.flush_overlays(&mut buf);
        }
        hits
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn a_secondary_click_on_a_row_selects_it_and_opens_at_the_pointer() {
        let mut l = list();
        let hits = frame(&mut l, None);
        let pos = Position::new(9, 7);
        let id = hits.hit(pos).expect("a row");
        let req = ContextConfig::DEFAULT
            .on_secondary(&mut l, id, pos)
            .expect("a row has a menu");
        assert_eq!(req.target, 5);
        assert_eq!(l.cursor, 5, "the row the menu is about is highlighted");
        let mut m = req.menu(MENU, items());
        frame(&mut l, Some(&mut m));
        assert_eq!(
            (m.area.x, m.area.y),
            (9, 8),
            "the menu opens at the pointer"
        );

        let mut still = list();
        frame(&mut still, None);
        let req = ContextConfig::DEFAULT
            .moves_cursor(false)
            .on_secondary(&mut still, id, pos)
            .unwrap();
        assert_eq!((req.target, still.cursor), (5, 0), "knob: cursor stays");
    }

    #[test]
    fn shift_f10_and_the_menu_key_open_below_the_cursor_row() {
        let mut l = list();
        l.cursor = 3;
        frame(&mut l, None);
        let cfg = ContextConfig::DEFAULT;
        assert!(
            cfg.on_key(&l, &key(KeyCode::F(10), KeyModifiers::NONE))
                .is_none()
        );
        assert!(
            cfg.on_key(&l, &key(KeyCode::Char('m'), KeyModifiers::NONE))
                .is_none()
        );
        let req = cfg
            .on_key(&l, &key(KeyCode::F(10), KeyModifiers::SHIFT))
            .expect("Shift+F10");
        assert_eq!(req.target, 3);
        assert_eq!(
            cfg.on_key(&l, &key(KeyCode::Menu, KeyModifiers::NONE)),
            Some(req),
            "the Menu key"
        );
        let mut m = req.menu(MENU, items());
        frame(&mut l, Some(&mut m));
        // the list starts at y = 2; row 3 is drawn at y = 5
        assert_eq!(m.area.y, 6, "the menu sits right under the cursor row");
        assert!(
            ContextConfig::DEFAULT
                .keys(ContextKeys::NONE)
                .on_key(&l, &key(KeyCode::F(10), KeyModifiers::SHIFT))
                .is_none(),
            "knob: keys off"
        );
        let extra = ContextKeys::NONE.extra(Some(key(KeyCode::Char('m'), KeyModifiers::NONE)));
        assert!(extra.matches(&key(KeyCode::Char('m'), KeyModifiers::NONE)));
    }

    #[test]
    fn a_secondary_click_outside_an_open_menu_closes_it_and_reopens_on_the_row() {
        let mut l = list();
        let mut m = ContextMenu::new(MENU, items()).at(Position::new(20, 2));
        let hits = frame(&mut l, Some(&mut m));
        let row = Position::new(3, 9);
        let mut p = PointerState::new(PointerConfig::DEFAULT);
        let SecondaryRoute::Outside { backdrop, reopen } = p.route_secondary(&hits, row) else {
            panic!("outside the menu");
        };
        assert!(m.owns(backdrop), "the click closes the open menu");
        assert_eq!(m.on_click(backdrop), Some(MenuAction::Dismissed));
        let below = reopen.expect("the row beneath");
        assert_eq!(l.locate(below), Some(7));
        assert_eq!(p.hover, Some(below));
        let inside = Position::new(m.area.x + 2, m.area.y + 1);
        assert!(matches!(
            p.route_secondary(&hits, inside),
            SecondaryRoute::Target(_)
        ));

        let mut close =
            PointerState::new(PointerConfig::DEFAULT.secondary_outside(SecondaryOutside::Close));
        assert!(matches!(
            close.route_secondary(&hits, row),
            SecondaryRoute::Outside { reopen: None, .. }
        ));
        let mut keep =
            PointerState::new(PointerConfig::DEFAULT.secondary_outside(SecondaryOutside::Ignore));
        assert_eq!(keep.route_secondary(&hits, row), SecondaryRoute::Swallowed);
    }

    #[test]
    fn tabs_name_their_cursor_tab() {
        let mut tabs = Tabs::new(WidgetId::of("ctx.tabs"), &["a", "b"]);
        tabs.cursor = 1;
        let req = ContextConfig::DEFAULT
            .on_key(&tabs, &key(KeyCode::F(10), KeyModifiers::SHIFT))
            .unwrap();
        assert_eq!(req.target, 1);
        assert!(matches!(req.anchor, ContextAnchor::Control { id, .. } if id == tabs.tab_id(1)));
        assert_eq!(tabs.context_target(tabs.tab_id(0)), Some(0));
    }
}
