//! Cursor stepping shared by list-like widgets.

/// Step `cursor` by `delta` in a list of `len` rows. `wrap`: past either
/// end lands on the other end (menus); otherwise it stops at the end
/// (lists, trees, tabs, selects).
pub fn step_index(cursor: usize, len: usize, delta: isize, wrap: bool) -> usize {
    if len == 0 {
        return 0;
    }
    let to = cursor as isize + delta;
    if wrap {
        to.rem_euclid(len as isize) as usize
    } else {
        to.clamp(0, len as isize - 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_clamp_or_wrap() {
        assert_eq!(step_index(0, 3, -1, false), 0);
        assert_eq!(step_index(2, 3, 1, false), 2);
        assert_eq!(step_index(0, 3, -1, true), 2);
        assert_eq!(step_index(2, 3, 1, true), 0);
        assert_eq!(step_index(1, 3, 1, true), 2);
        assert_eq!(step_index(0, 0, 1, true), 0);
    }

    use crate::core::event::Key;
    use crate::core::id::WidgetId;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn k(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    #[test]
    fn every_list_like_widget_takes_a_wrap_knob_with_its_old_default() {
        use crate::widgets::list::{ListBox, ListItem, SelectMode};
        use crate::widgets::menu::{ContextMenu, MenuBar, MenuItem};
        use crate::widgets::select::Select;
        use crate::widgets::tabs::Tabs;
        use crate::widgets::tree::{TreeNode, TreeView};

        let items = || vec![ListItem::new("a"), ListItem::new("b"), ListItem::new("c")];
        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Single);
        l.on_key(&k(KeyCode::Up));
        assert_eq!(l.cursor, 0, "a list clamps by default");
        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Single).wrap(true);
        l.on_key(&k(KeyCode::Up));
        assert_eq!(l.cursor, 2, "and wraps on request");

        let nodes = || vec![TreeNode::leaf("a"), TreeNode::leaf("b")];
        let mut t = TreeView::new(WidgetId::of("t"), nodes());
        t.on_key(&k(KeyCode::Up));
        assert_eq!(t.cursor, 0);
        let mut t = TreeView::new(WidgetId::of("t"), nodes()).wrap(true);
        t.on_key(&k(KeyCode::Up));
        assert_eq!(t.cursor, 1);

        let mut tabs = Tabs::new(WidgetId::of("tabs"), &["a", "b", "c"]);
        tabs.on_key(&k(KeyCode::Left));
        assert_eq!(tabs.active, 0);
        let mut tabs = Tabs::new(WidgetId::of("tabs"), &["a", "b", "c"]).wrap(true);
        tabs.on_key(&k(KeyCode::Left));
        assert_eq!(tabs.active, 2);

        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 2);
        s.on_key(&k(KeyCode::Down));
        assert_eq!(s.selected, 2);
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 2).wrap(true);
        s.on_key(&k(KeyCode::Down));
        assert_eq!(s.selected, 0);

        let menu = || vec![MenuItem::new("a"), MenuItem::new("b")];
        let mut m = ContextMenu::new(WidgetId::of("m"), menu());
        m.on_key(&k(KeyCode::Up));
        assert_eq!(m.cursor, 1, "a menu wraps by default");
        let mut m = ContextMenu::new(WidgetId::of("m"), menu()).wrap(false);
        m.on_key(&k(KeyCode::Up));
        assert_eq!(m.cursor, 0, "and clamps on request");

        let mut bar = MenuBar::new(WidgetId::of("bar"), vec![("A", menu()), ("B", menu())]);
        bar.on_key(&k(KeyCode::Left));
        assert_eq!(bar.cursor, 1);
        let mut bar =
            MenuBar::new(WidgetId::of("bar"), vec![("A", menu()), ("B", menu())]).wrap(false);
        bar.on_key(&k(KeyCode::Left));
        assert_eq!(bar.cursor, 0);
    }
}
