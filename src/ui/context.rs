use crate::core::keymap::Actions;
use crate::core::list::Navigable;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::core::event::Key;
use crate::core::id::{Part, WidgetId};
use crate::core::pointer::PointerEvent;
use crate::widgets::grid::DataGrid;
use crate::widgets::list::ListBox;
use crate::widgets::menu::{ContextMenu, MenuItem};
use crate::widgets::table::DataTable;
use crate::widgets::tabs::Tabs;
use crate::widgets::tree::TreeView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextKeys {
    pub shift_f10: bool,

    pub menu_key: bool,

    pub extra: Option<Key>,
}

impl ContextKeys {
    pub const DEFAULT: Self = Self {
        shift_f10: true,
        menu_key: true,
        extra: None,
    };

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextConfig {
    pub keys: ContextKeys,

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

    pub fn on_secondary<S: ContextSource + ?Sized>(
        &self,
        src: &mut S,
        ev: &PointerEvent,
    ) -> Option<ContextRequest<S::Target>> {
        let target = src.context_target(ev)?;
        if self.moves_cursor {
            src.context_select(target);
        }
        Some(ContextRequest {
            target,
            anchor: ContextAnchor::Pointer(ev.pos()),
        })
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAnchor {
    Pointer(Position),

    Control { id: WidgetId, fallback: Rect },
}

impl ContextAnchor {
    pub fn apply(self, menu: ContextMenu) -> ContextMenu {
        match self {
            Self::Pointer(pos) => menu.at(pos),
            Self::Control { id, fallback } => menu.anchor_control(id, fallback),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextRequest<T> {
    pub target: T,
    pub anchor: ContextAnchor,
}

impl<T> ContextRequest<T> {
    pub fn menu(&self, id: WidgetId, items: Vec<MenuItem>) -> ContextMenu {
        self.anchor.apply(ContextMenu::new(id, items))
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ContextRequest<U> {
        ContextRequest {
            target: f(self.target),
            anchor: self.anchor,
        }
    }
}

pub trait ContextSource {
    type Target: Copy;

    fn context_target(&self, ev: &PointerEvent) -> Option<Self::Target>;

    fn context_select(&mut self, target: Self::Target);

    fn context_cursor(&self) -> Option<ContextRequest<Self::Target>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextCmd<A> {
    pub cmd: A,
    pub reason: Option<&'static str>,
    pub group_end: bool,
    pub danger: bool,
}

impl<A> ContextCmd<A> {
    pub const fn new(cmd: A) -> Self {
        Self {
            cmd,
            reason: None,
            group_end: false,
            danger: false,
        }
    }

    pub const fn unless(mut self, reason: Option<&'static str>) -> Self {
        self.reason = reason;
        self
    }

    pub const fn group_end(mut self) -> Self {
        self.group_end = true;
        self
    }

    pub const fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
}

pub fn command_items<A: Actions + Copy + PartialEq>(
    map: &[(crate::core::keymap::KeyBinding, A)],
    cmds: &[ContextCmd<A>],
) -> Vec<(A, MenuItem)> {
    cmds.iter()
        .map(|c| {
            let mut item = MenuItem::new(c.cmd.label())
                .separator_if(c.group_end)
                .danger_if(c.danger);
            if let Some(k) = crate::core::keymap::label_for(map, c.cmd) {
                item = item.shortcut(k);
            }
            if let Some(r) = c.reason {
                item = item.reason(r);
            }
            (c.cmd, item)
        })
        .collect()
}

fn row_rect(area: Rect, row: usize, offset: usize) -> Rect {
    let dy = row
        .saturating_sub(offset)
        .min(area.height.saturating_sub(1) as usize) as u16;
    Rect::new(area.x, area.y + dy, area.width.max(1), 1)
}

impl ContextSource for ListBox {
    type Target = usize;
    fn context_target(&self, ev: &PointerEvent) -> Option<usize> {
        ev.on(self.id).then(|| ev.index_of(Part::ROW))?
    }
    fn context_select(&mut self, row: usize) {
        if let Some(i) = Navigable::cursor(self).select(row, self).index() {
            self.cursor = i;
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
    fn context_target(&self, ev: &PointerEvent) -> Option<usize> {
        self.row_at(ev)
    }
    fn context_select(&mut self, row: usize) {
        if let Some(i) = Navigable::cursor(self).select(row, self).index() {
            self.cursor = i;
        }
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
    type Target = (usize, usize);
    fn context_target(&self, ev: &PointerEvent) -> Option<(usize, usize)> {
        if !ev.on(self.id) {
            return None;
        }
        if let Some(r) = ev.index_of(Part::ROW) {
            return Some((r, self.cursor_col));
        }
        let r = ev.index_of(Part::CELL)?;
        Some((r, self.column_at(ev.pos().x).unwrap_or(self.cursor_col)))
    }

    fn context_select(&mut self, (row, col): (usize, usize)) {
        if let Some(r) = Navigable::cursor(self).select(row, self).index() {
            self.cursor_row = r;
            if self.cell_nav && col < self.columns.len() {
                self.cursor_col = col;
            }
            self.context_selection(r);
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
    type Target = (usize, usize);
    fn context_target(&self, ev: &PointerEvent) -> Option<(usize, usize)> {
        if !ev.on(self.id) {
            return None;
        }
        if let Some(r) = ev.index_of(Part::ROW_NUMBER) {
            return Some((r, self.cursor.1));
        }
        let r = ev.index_of(Part::CELL)?;
        Some((r, self.column_at(ev.pos().x)?))
    }

    fn context_select(&mut self, cell: (usize, usize)) {
        if cell.1 < self.columns.len()
            && let Some(r) = Navigable::cursor(self).select(cell.0, self).index()
        {
            self.context_selection(r, cell.1);
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
    fn context_target(&self, ev: &PointerEvent) -> Option<usize> {
        ev.on(self.id).then(|| ev.index_of(Part::TAB))?
    }
    fn context_select(&mut self, tab: usize) {
        if let Some(i) = Navigable::cursor(self).select(tab, self).index() {
            self.cursor = i;
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

    fn secondary(hits: &HitRegistry, id: WidgetId, pos: Position) -> PointerEvent {
        PointerEvent::new(
            hits,
            crate::core::intent::Phase::Secondary,
            id,
            pos,
            1,
            KeyModifiers::NONE,
        )
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn context_select_goes_through_the_cursor_and_ignores_stale_targets() {
        use crate::widgets::tree::{TreeNode, TreeView};

        let mut t = TreeView::new(
            WidgetId::of("t"),
            vec![TreeNode::leaf("a"), TreeNode::leaf("b")],
        );
        t.context_select(1);
        assert_eq!(t.cursor, 1);
        t.context_select(7);
        assert_eq!(t.cursor, 1, "out of range is ignored");
        assert!(t.context_cursor().is_some());
        let mut l = ListBox::new(
            WidgetId::of("l"),
            vec![ListItem::new("a"), ListItem::new("b").disabled(true)],
            SelectMode::Single,
        );
        l.context_select(1);
        assert_eq!(l.cursor, 1, "a pointer may target a disabled row");
        l.context_select(2);
        assert_eq!(l.cursor, 1);
    }

    #[test]
    fn a_right_click_outside_the_selection_selects_only_that_row() {
        use crate::core::selection::SelectionMode;
        use crate::widgets::table::{Cell, Column, DataTable};
        let rows = (0..5).map(|i| vec![Cell::new(format!("r{i}"))]).collect();
        let cols = vec![Column::new("Name", ratatui::layout::Constraint::Min(4))];
        let mut t =
            DataTable::new(WidgetId::of("t"), cols, rows).selection_mode(SelectionMode::Multi);
        let p = Position::new(0, 0);
        t.on_click_cell_with(1, 0, p, KeyModifiers::NONE);
        t.on_click_cell_with(3, 0, p, KeyModifiers::SHIFT);
        assert_eq!(t.selected_rows(), vec![1, 2, 3]);
        t.context_select((2, 0));
        assert_eq!(t.selected_rows(), vec![1, 2, 3], "inside: kept");
        assert_eq!(t.cursor_row, 2);
        t.context_select((4, 0));
        assert_eq!(t.selected_rows(), vec![4], "outside: only that row");
        assert_eq!(t.cursor_row, 4);
    }
    #[test]
    fn a_secondary_click_on_a_row_selects_it_and_opens_at_the_pointer() {
        let mut l = list();
        let hits = frame(&mut l, None);
        let pos = Position::new(9, 7);
        let id = hits.hit(pos).expect("a row");
        let ev = secondary(&hits, id, pos);
        let req = ContextConfig::DEFAULT
            .on_secondary(&mut l, &ev)
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
            .on_secondary(&mut still, &ev)
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
        assert_eq!(
            hits.part_of(backdrop).map(|h| h.owner),
            Some(MENU),
            "the click closes the open menu"
        );
        assert_eq!(m.on_click(backdrop), Some(MenuAction::Dismissed));
        let below = reopen.expect("the row beneath");
        assert_eq!(l.context_target(&secondary(&hits, below, row)), Some(7));
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
        let mut hits = HitRegistry::default();
        let tab = hits.register_part(
            tabs.id,
            crate::core::id::PartRef::item(Part::TAB, crate::core::id::ItemKey::index(0)),
            Rect::new(0, 0, 4, 1),
        );
        assert_eq!(tab, tabs.tab_id(0));
        let ev = secondary(&hits, tab, Position::new(1, 0));
        assert_eq!(tabs.context_target(&ev), Some(0));
    }
}
