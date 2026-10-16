//! Key maps shared by every navigable widget.
//!
//! Two layers:
//!
//! - [`NavKeys`] decides which keys *move* (arrows, vim letters, paging,
//!   Home/End). Every navigable widget asks [`NavKeys::action`] instead of
//!   matching key codes itself, so a single setting (`vim: false`, say)
//!   changes List, Tree, Table, Grid, Select, menus and the rest alike.
//! - [`KeyMap<A>`] binds a widget's own *actions* (chips `x`, tree `*`,
//!   Grid's letters) to keys. Each action enum implements [`Actions`], whose
//!   [`Actions::default_keymap`] reproduces the long-standing letters; a
//!   widget's `.keymap(..)` builder replaces it. Hints that name a key read
//!   it back with [`label_for`], so a remapped key is advertised correctly.

use crate::core::event::Key;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

/// A navigation step, independent of the key that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Which keys navigate. One table for every widget.
///
/// | key | `Nav` | enabled by |
/// |---|---|---|
/// | `↑ ↓ ← →` (any modifiers) | `Up Down Left Right` | `arrows` |
/// | `k j h l` and `K J H L` (no Ctrl/Alt) | `Up Down Left Right` | `vim` |
/// | `PageUp PageDown` (any modifiers) | `PageUp PageDown` | `page` |
/// | `Home End` (any modifiers) | `Home End` | `home_end` |
/// | `g` / `G` (no Ctrl/Alt) | `Home` / `End` | `vim` |
///
/// Named keys come back whatever modifiers are held, so a widget can still
/// treat Shift+↓ as "extend" or Ctrl+Home as "first cell"; letters never
/// match with Ctrl or Alt held, so `Ctrl+K` is free for an app. The
/// uppercase vim letters are the Shift variants (`J` extends like Shift+↓
/// where a widget extends).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavKeys {
    /// The arrow keys.
    pub arrows: bool,
    /// `h j k l` (and uppercase) plus `g`/`G` for first/last.
    pub vim: bool,
    /// PageUp / PageDown.
    pub page: bool,
    /// Home / End.
    pub home_end: bool,
}

impl NavKeys {
    /// Today's behaviour: everything on, vim letters included.
    pub const DEFAULT: Self = Self {
        arrows: true,
        vim: true,
        page: true,
        home_end: true,
    };

    /// Desktop keys only: arrows, paging and Home/End; letters stay free
    /// for typing or app shortcuts.
    pub const NO_VIM: Self = Self {
        vim: false,
        ..Self::DEFAULT
    };

    /// See [`Self::vim`].
    pub const fn vim(mut self, on: bool) -> Self {
        self.vim = on;
        self
    }

    /// See [`Self::page`].
    pub const fn page(mut self, on: bool) -> Self {
        self.page = on;
        self
    }

    /// See [`Self::home_end`].
    pub const fn home_end(mut self, on: bool) -> Self {
        self.home_end = on;
        self
    }

    /// See [`Self::arrows`].
    pub const fn arrows(mut self, on: bool) -> Self {
        self.arrows = on;
        self
    }

    /// The navigation step `key` means, or `None` when it does not
    /// navigate under this configuration.
    pub fn action(&self, key: &Key) -> Option<Nav> {
        let nav = match key.code {
            KeyCode::Up if self.arrows => Nav::Up,
            KeyCode::Down if self.arrows => Nav::Down,
            KeyCode::Left if self.arrows => Nav::Left,
            KeyCode::Right if self.arrows => Nav::Right,
            KeyCode::PageUp if self.page => Nav::PageUp,
            KeyCode::PageDown if self.page => Nav::PageDown,
            KeyCode::Home if self.home_end => Nav::Home,
            KeyCode::End if self.home_end => Nav::End,
            KeyCode::Char(c) if self.vim && !key.ctrl() && !key.alt() => match c {
                'k' | 'K' => Nav::Up,
                'j' | 'J' => Nav::Down,
                'h' | 'H' => Nav::Left,
                'l' | 'L' => Nav::Right,
                'g' => Nav::Home,
                'G' => Nav::End,
                _ => return None,
            },
            _ => return None,
        };
        Some(nav)
    }

    /// True when `key` is one of the vim letters this table maps (whatever
    /// `vim` is set to). Widgets with type-ahead use it to tell a letter
    /// that would navigate from one that only types.
    pub fn is_vim_letter(key: &Key) -> bool {
        matches!(
            key.code,
            KeyCode::Char('k' | 'K' | 'j' | 'J' | 'h' | 'H' | 'l' | 'L' | 'g' | 'G')
        )
    }
}

impl Default for NavKeys {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One key chord a [`KeyMap`] binds.
///
/// For a character the Shift modifier is part of the character (`X` is
/// Shift+x), so it is ignored when matching: `KeyBinding::char('X')`
/// matches `X` whether or not the terminal reports Shift. Every other
/// modifier must match exactly, so `KeyBinding::char('x')` never fires on
/// Ctrl+X.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl KeyBinding {
    pub const fn new(code: KeyCode, mods: KeyModifiers) -> Self {
        Self { code, mods }
    }

    /// `code` with no modifier.
    pub const fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    /// A plain character.
    pub const fn char(c: char) -> Self {
        Self::key(KeyCode::Char(c))
    }

    /// Ctrl + a character.
    pub const fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Alt + a character.
    pub const fn alt(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    /// True when `key` is this chord.
    pub fn matches(&self, key: &Key) -> bool {
        if self.code != key.code {
            return false;
        }
        if matches!(self.code, KeyCode::Char(_)) {
            self.mods.difference(KeyModifiers::SHIFT) == key.mods.difference(KeyModifiers::SHIFT)
        } else {
            self.mods == key.mods
        }
    }

    /// How hints spell this chord: `x`, `Ctrl+S`, `Delete`, `F2`, `Space`.
    pub fn label(&self) -> String {
        let mut out = String::new();
        if self.mods.contains(KeyModifiers::CONTROL) {
            out.push_str("Ctrl+");
        }
        if self.mods.contains(KeyModifiers::ALT) {
            out.push_str("Alt+");
        }
        let ctrl_or_alt = self
            .mods
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        if self.mods.contains(KeyModifiers::SHIFT) && !matches!(self.code, KeyCode::Char(_)) {
            out.push_str("Shift+");
        }
        match self.code {
            KeyCode::Char(' ') => out.push_str("Space"),
            KeyCode::Char(c) if ctrl_or_alt => out.extend(c.to_uppercase()),
            KeyCode::Char(c) => out.push(c),
            KeyCode::Enter => out.push_str("Enter"),
            KeyCode::Esc => out.push_str("Esc"),
            KeyCode::Tab => out.push_str("Tab"),
            KeyCode::BackTab => out.push_str("Shift+Tab"),
            KeyCode::Backspace => out.push_str("Backspace"),
            KeyCode::Delete => out.push_str("Delete"),
            KeyCode::Insert => out.push_str("Ins"),
            KeyCode::Up => out.push('↑'),
            KeyCode::Down => out.push('↓'),
            KeyCode::Left => out.push('←'),
            KeyCode::Right => out.push('→'),
            KeyCode::Home => out.push_str("Home"),
            KeyCode::End => out.push_str("End"),
            KeyCode::PageUp => out.push_str("PgUp"),
            KeyCode::PageDown => out.push_str("PgDn"),
            KeyCode::F(n) => out.push_str(&format!("F{n}")),
            other => out.push_str(&format!("{other:?}")),
        }
        out
    }
}

impl From<KeyCode> for KeyBinding {
    fn from(code: KeyCode) -> Self {
        Self::key(code)
    }
}

impl From<char> for KeyBinding {
    fn from(c: char) -> Self {
        Self::char(c)
    }
}

/// A widget's action bindings, first match wins. Several keys may bind the
/// same action; an action with no entry is unbound.
pub type KeyMap<A> = Vec<(KeyBinding, A)>;

/// A widget's action enum.
pub trait Actions: Copy + PartialEq + Sized + 'static {
    /// The bindings the widget has always had.
    fn default_keymap() -> KeyMap<Self>;
}

/// The action `key` is bound to in `map`.
pub fn lookup<A: Copy>(map: &[(KeyBinding, A)], key: &Key) -> Option<A> {
    map.iter().find(|(b, _)| b.matches(key)).map(|&(_, a)| a)
}

/// The first key bound to `action`, if any.
pub fn binding_for<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<KeyBinding> {
    map.iter().find(|(_, a)| *a == action).map(|&(b, _)| b)
}

/// The hint label of the first key bound to `action`, if any.
pub fn label_for<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<String> {
    binding_for(map, action).map(|b| b.label())
}

/// [`label_for`] as a `&'static str`, for [`crate::widgets::keyhint::Hint`]
/// keys: each distinct chord's label is interned once, so a footer built
/// every frame names the bound key (or drops the hint when the action is
/// unbound) without allocating.
pub fn hint_key<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<&'static str> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static LABELS: OnceLock<Mutex<HashMap<KeyBinding, &'static str>>> = OnceLock::new();
    let b = binding_for(map, action)?;
    let mut labels = LABELS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Some(
        labels
            .entry(b)
            .or_insert_with(|| Box::leak(b.label().into_boxed_str())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn with(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn the_shared_nav_table() {
        let d = NavKeys::DEFAULT;
        let table = [
            (KeyCode::Up, Some(Nav::Up)),
            (KeyCode::Down, Some(Nav::Down)),
            (KeyCode::Left, Some(Nav::Left)),
            (KeyCode::Right, Some(Nav::Right)),
            (KeyCode::PageUp, Some(Nav::PageUp)),
            (KeyCode::PageDown, Some(Nav::PageDown)),
            (KeyCode::Home, Some(Nav::Home)),
            (KeyCode::End, Some(Nav::End)),
            (KeyCode::Char('k'), Some(Nav::Up)),
            (KeyCode::Char('K'), Some(Nav::Up)),
            (KeyCode::Char('j'), Some(Nav::Down)),
            (KeyCode::Char('J'), Some(Nav::Down)),
            (KeyCode::Char('h'), Some(Nav::Left)),
            (KeyCode::Char('H'), Some(Nav::Left)),
            (KeyCode::Char('l'), Some(Nav::Right)),
            (KeyCode::Char('L'), Some(Nav::Right)),
            (KeyCode::Char('g'), Some(Nav::Home)),
            (KeyCode::Char('G'), Some(Nav::End)),
            (KeyCode::Char('x'), None),
            (KeyCode::Enter, None),
            (KeyCode::Tab, None),
        ];
        for (code, want) in table {
            assert_eq!(d.action(&key(code)), want, "{code:?}");
        }
    }

    #[test]
    fn named_keys_keep_their_modifiers_and_letters_refuse_ctrl_alt() {
        let d = NavKeys::DEFAULT;
        assert_eq!(
            d.action(&with(KeyCode::Down, KeyModifiers::SHIFT)),
            Some(Nav::Down)
        );
        assert_eq!(
            d.action(&with(KeyCode::Home, KeyModifiers::CONTROL)),
            Some(Nav::Home)
        );
        assert_eq!(
            d.action(&with(KeyCode::Char('J'), KeyModifiers::SHIFT)),
            Some(Nav::Down)
        );
        assert_eq!(
            d.action(&with(KeyCode::Char('k'), KeyModifiers::CONTROL)),
            None
        );
        assert_eq!(d.action(&with(KeyCode::Char('j'), KeyModifiers::ALT)), None);
    }

    #[test]
    fn each_group_switches_off_on_its_own() {
        let no_vim = NavKeys::NO_VIM;
        for c in ['h', 'j', 'k', 'l', 'g', 'G', 'J', 'K'] {
            assert_eq!(no_vim.action(&key(KeyCode::Char(c))), None, "{c}");
        }
        assert_eq!(no_vim.action(&key(KeyCode::Down)), Some(Nav::Down));
        let no_page = NavKeys::DEFAULT.page(false);
        assert_eq!(no_page.action(&key(KeyCode::PageDown)), None);
        assert_eq!(no_page.action(&key(KeyCode::Char('j'))), Some(Nav::Down));
        let no_home = NavKeys::DEFAULT.home_end(false);
        assert_eq!(no_home.action(&key(KeyCode::Home)), None);
        assert_eq!(no_home.action(&key(KeyCode::Char('g'))), Some(Nav::Home));
        let no_arrows = NavKeys::DEFAULT.arrows(false);
        assert_eq!(no_arrows.action(&key(KeyCode::Up)), None);
        assert_eq!(NavKeys::default(), NavKeys::DEFAULT);
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Act {
        Remove,
        Add,
    }

    impl Actions for Act {
        fn default_keymap() -> KeyMap<Self> {
            vec![
                (KeyBinding::char('x'), Act::Remove),
                (KeyBinding::key(KeyCode::Delete), Act::Remove),
                (KeyBinding::char('+'), Act::Add),
            ]
        }
    }

    #[test]
    fn bindings_match_lookup_and_label() {
        let map = Act::default_keymap();
        assert_eq!(lookup(&map, &key(KeyCode::Char('x'))), Some(Act::Remove));
        assert_eq!(lookup(&map, &key(KeyCode::Delete)), Some(Act::Remove));
        assert_eq!(
            lookup(&map, &with(KeyCode::Char('+'), KeyModifiers::SHIFT)),
            Some(Act::Add),
            "Shift is part of the character"
        );
        assert_eq!(
            lookup(&map, &with(KeyCode::Char('x'), KeyModifiers::CONTROL)),
            None
        );
        assert_eq!(
            lookup(&map, &with(KeyCode::Delete, KeyModifiers::SHIFT)),
            None
        );
        assert_eq!(label_for(&map, Act::Remove).as_deref(), Some("x"));
        assert_eq!(KeyBinding::ctrl('s').label(), "Ctrl+S");
        assert_eq!(KeyBinding::key(KeyCode::Delete).label(), "Delete");
        assert_eq!(hint_key(&map, Act::Add), Some("+"));
        assert_eq!(
            hint_key(&map, Act::Add).map(str::as_ptr),
            hint_key(&map, Act::Add).map(str::as_ptr),
            "labels are interned once"
        );
        assert_eq!(KeyBinding::char(' ').label(), "Space");
        let remapped: KeyMap<Act> = vec![(KeyBinding::char('d'), Act::Remove)];
        assert_eq!(lookup(&remapped, &key(KeyCode::Char('x'))), None);
        assert_eq!(binding_for(&remapped, Act::Add), None);
    }
}

/// Every navigable widget reads [`NavKeys`] and its [`KeyMap`]: a remapped
/// binding fires, the default it replaced does not, and `NavKeys::NO_VIM`
/// frees the letters.
#[cfg(test)]
mod widget_tests {
    use super::*;
    use crate::core::event::Outcome;
    use crate::core::id::WidgetId;

    fn k(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn c(ch: char) -> Key {
        k(KeyCode::Char(ch))
    }

    #[test]
    fn list_nav_and_keymap() {
        use crate::widgets::list::{ListAction, ListBox, ListItem, SelectMode};
        let items = || vec![ListItem::new("a"), ListItem::new("b"), ListItem::new("c")];
        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Single);
        l.on_key(&c('j'));
        assert_eq!(l.cursor, 1, "vim on by default");
        let mut l =
            ListBox::new(WidgetId::of("l"), items(), SelectMode::Single).nav(NavKeys::NO_VIM);
        assert_eq!(l.on_key(&c('j')), Outcome::Ignored);
        assert_eq!(l.cursor, 0);
        l.on_key(&k(KeyCode::Down));
        assert_eq!(l.cursor, 1, "arrows stay");

        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Multi);
        l.on_key(&c('a'));
        assert!(l.checked.iter().all(|c| *c), "`a` checks all by default");
        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Multi)
            .keymap(vec![(KeyBinding::ctrl('a'), ListAction::ToggleAll)]);
        assert_eq!(
            l.on_key(&c('a')),
            Outcome::Ignored,
            "the default is unbound"
        );
        l.on_key(&Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::CONTROL,
        });
        assert!(l.checked.iter().all(|c| *c), "the remapped key fires");
    }

    #[test]
    fn tree_nav_and_keymap() {
        use crate::widgets::tree::{TreeAction, TreeNode, TreeView};
        let nodes = || {
            vec![
                TreeNode::dir("a", vec![TreeNode::dir("a1", vec![TreeNode::leaf("x")])]),
                TreeNode::leaf("b"),
            ]
        };
        let mut t = TreeView::new(WidgetId::of("t"), nodes());
        t.on_key(&c('J'));
        assert_eq!(t.cursor, 1, "J moves like j, as in List and Grid");
        t.on_key(&c('K'));
        assert_eq!(t.cursor, 0);
        let mut t = TreeView::new(WidgetId::of("t"), nodes()).nav(NavKeys::NO_VIM);
        assert_eq!(t.on_key(&c('j')).0, Outcome::Ignored);
        assert_eq!(t.cursor, 0);

        let mut t = TreeView::new(WidgetId::of("t"), nodes());
        let rows = t.rows().len();
        t.on_key(&c('*'));
        assert!(t.rows().len() > rows, "`*` expands all by default");
        let mut t = TreeView::new(WidgetId::of("t"), nodes())
            .keymap(vec![(KeyBinding::char('e'), TreeAction::ExpandAll)]);
        let rows = t.rows().len();
        assert_eq!(
            t.on_key(&c('*')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        t.on_key(&c('e'));
        assert!(t.rows().len() > rows, "the remapped key fires");
        let expanded = t.rows().len();
        t.on_key(&c('-'));
        assert_eq!(
            t.rows().len(),
            expanded,
            "an action left out of the map is unbound"
        );
    }

    #[test]
    fn table_nav_and_keymap() {
        use crate::widgets::table::{Cell, Column, DataTable, TableAction};
        use ratatui::layout::Constraint;
        let table = || {
            DataTable::new(
                WidgetId::of("t"),
                vec![Column::new("n", Constraint::Length(4))],
                vec![vec![Cell::new("b")], vec![Cell::new("a")]],
            )
        };
        let mut t = table();
        t.on_key(&c('j'));
        assert_eq!(t.cursor_row, 1);
        let mut t = table().nav(NavKeys::NO_VIM);
        assert_eq!(t.on_key(&c('j')).0, Outcome::Ignored);
        assert_eq!(t.cursor_row, 0);

        let mut t = table();
        t.on_key(&c('s'));
        assert!(t.sort.is_some(), "`s` sorts by default");
        let mut t = table().keymap(vec![(KeyBinding::char('o'), TableAction::SortByColumn)]);
        assert_eq!(
            t.on_key(&c('s')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert!(t.sort.is_none());
        t.on_key(&c('o'));
        assert!(t.sort.is_some(), "the remapped key fires");
    }

    #[test]
    fn select_nav() {
        use crate::widgets::select::Select;
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0);
        s.on_key(&k(KeyCode::Enter));
        s.on_key(&c('G'));
        assert_eq!(s.cursor, 2, "the open list takes the whole table (G = End)");
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0).nav(NavKeys::NO_VIM);
        s.on_key(&k(KeyCode::Enter));
        s.on_key(&c('j'));
        assert_eq!(s.cursor, 0);
        s.on_key(&k(KeyCode::Down));
        assert_eq!(s.cursor, 1);
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0);
        assert_eq!(
            s.on_key(&c('j')).0,
            Outcome::Ignored,
            "a closed field leaves letters to the page"
        );
    }

    #[test]
    fn menus_nav() {
        use crate::widgets::menu::{ContextMenu, MenuBar, MenuItem};
        let menu = || {
            (0..20)
                .map(|i| MenuItem::new(format!("item {i}")))
                .collect::<Vec<_>>()
        };
        let mut m = ContextMenu::new(WidgetId::of("m"), menu());
        m.on_key(&c('j'));
        assert_eq!(m.cursor, 1);
        let mut m = ContextMenu::new(WidgetId::of("m"), menu()).nav(NavKeys::NO_VIM);
        m.on_key(&c('j'));
        assert_eq!(m.cursor, 0);
        m.on_key(&k(KeyCode::Down));
        assert_eq!(m.cursor, 1);

        let mut bar = MenuBar::new(WidgetId::of("bar"), vec![("A", menu()), ("B", menu())])
            .nav(NavKeys::NO_VIM);
        bar.on_key(&c('l'));
        assert_eq!(bar.cursor, 0);
        bar.on_key(&k(KeyCode::Right));
        assert_eq!(bar.cursor, 1);
        bar.on_key(&k(KeyCode::Enter));
        let open = bar.open.as_ref().expect("menu opens");
        assert_eq!(
            open.nav,
            NavKeys::NO_VIM,
            "the open menu inherits the bar's keys"
        );
    }

    #[test]
    fn radio_and_steps_nav() {
        use crate::widgets::choice::RadioGroup;
        use crate::widgets::steps::{Step, StepRail};
        let mut r = RadioGroup::new(WidgetId::of("r"), "R", &["a", "b", "c"], 0);
        r.on_key(&c('j'));
        assert_eq!(r.selected, 1);
        r.on_key(&k(KeyCode::End));
        assert_eq!(
            r.selected, 2,
            "End picks the last choice, as in the other lists"
        );
        let mut r =
            RadioGroup::new(WidgetId::of("r"), "R", &["a", "b", "c"], 0).nav(NavKeys::NO_VIM);
        assert_eq!(r.on_key(&c('j')), Outcome::Ignored);
        assert_eq!(r.on_key(&k(KeyCode::Down)), Outcome::Changed);
        assert_eq!(r.selected, 1);

        let steps = || vec![Step::new("a"), Step::new("b"), Step::new("c")];
        let mut s = StepRail::new(WidgetId::of("s"), steps()).selectable(true);
        s.on_key(&c('j'));
        assert_eq!(s.cursor, 1);
        let mut s = StepRail::new(WidgetId::of("s"), steps())
            .selectable(true)
            .nav(NavKeys::NO_VIM);
        assert_eq!(s.on_key(&c('j')), Outcome::Ignored);
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn textarea_and_panel_nav_and_keymap() {
        use crate::widgets::panel::{PanelAction, ScrollPanel};
        use crate::widgets::textarea::TextArea;
        let mut t = TextArea::new(WidgetId::of("t"), "T", 3);
        assert_ne!(t.on_key(&c('j')).0, Outcome::Ignored);
        let mut t = TextArea::new(WidgetId::of("t"), "T", 3).nav(NavKeys::NO_VIM);
        assert_eq!(t.on_key(&c('j')).0, Outcome::Ignored);
        assert_ne!(t.on_key(&k(KeyCode::Down)).0, Outcome::Ignored);

        let lines = || (0..50).map(|i| format!("line {i}")).collect::<Vec<_>>();
        let mut p = ScrollPanel::new(WidgetId::of("p"), lines());
        assert_ne!(p.on_key(&c('j')), Outcome::Ignored);
        let mut p = ScrollPanel::new(WidgetId::of("p"), lines()).nav(NavKeys::NO_VIM);
        assert_eq!(p.on_key(&c('j')), Outcome::Ignored);

        let mut p = ScrollPanel::new(WidgetId::of("p"), lines()).tail(true);
        p.on_key(&c('f'));
        assert!(!p.follow, "`f` pauses following by default");
        let mut p = ScrollPanel::new(WidgetId::of("p"), lines())
            .tail(true)
            .keymap(vec![(KeyBinding::char('t'), PanelAction::Follow)]);
        assert_eq!(
            p.on_key(&c('f')),
            Outcome::Ignored,
            "the default is unbound"
        );
        assert!(p.follow);
        p.on_key(&c('t'));
        assert!(!p.follow, "the remapped key fires");
    }

    #[test]
    fn viewport_nav_and_keymap() {
        use crate::widgets::viewport::{TextViewport, ViewportAction};
        let mut v = TextViewport::new(WidgetId::of("v"));
        assert_ne!(v.on_key(&c('k')).0, Outcome::Ignored);
        let mut v = TextViewport::new(WidgetId::of("v")).nav(NavKeys::NO_VIM);
        assert_eq!(v.on_key(&c('k')).0, Outcome::Ignored);

        let mut v = TextViewport::new(WidgetId::of("v"));
        assert!(v.follow);
        v.on_key(&c('f'));
        assert!(!v.follow, "`f` toggles following by default");
        let mut v = TextViewport::new(WidgetId::of("v"))
            .keymap(vec![(KeyBinding::char('p'), ViewportAction::ToggleFollow)]);
        assert_eq!(
            v.on_key(&c('f')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert_eq!(v.on_key(&c('y')).0, Outcome::Ignored, "so is the copy key");
        assert!(v.follow);
        v.on_key(&c('p'));
        assert!(!v.follow, "the remapped key fires");
    }

    #[test]
    fn code_editor_nav_and_keymap() {
        use crate::widgets::code::{CodeAction, CodeEditor};
        let text = "a\nb\nc\n";
        let mut e = CodeEditor::new(WidgetId::of("e"), text);
        e.on_key(&c('j'));
        assert_eq!(e.buffer.cursor_pos().line, 1);
        let mut e = CodeEditor::new(WidgetId::of("e"), text).nav(NavKeys::NO_VIM);
        assert_eq!(e.on_key(&c('j')).0, Outcome::Ignored);
        assert_eq!(e.buffer.cursor_pos().line, 0);

        let mut e = CodeEditor::new(WidgetId::of("e"), text);
        e.on_key(&c('/'));
        assert!(e.find.is_some(), "`/` opens find by default");
        let mut e = CodeEditor::new(WidgetId::of("e"), text)
            .keymap(vec![(KeyBinding::ctrl('f'), CodeAction::Find)]);
        assert_eq!(
            e.on_key(&c('/')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert_eq!(e.on_key(&c('i')).0, Outcome::Ignored, "so is `i`");
        assert!(!e.editing);
        e.on_key(&Key {
            code: KeyCode::Char('f'),
            mods: KeyModifiers::CONTROL,
        });
        assert!(e.find.is_some(), "the remapped key fires");
    }

    fn grid() -> crate::widgets::grid::DataGrid {
        use crate::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
        let cols = vec![
            ColumnSpec::new("id", CellKind::Id).primary(),
            ColumnSpec::new("name", CellKind::Text),
        ];
        let mut g = DataGrid::new(WidgetId::of("g"), cols);
        let rows = (0..10)
            .map(|i| {
                vec![
                    CellValue::Text(format!("id-{i}")),
                    CellValue::Text(format!("row {i}")),
                ]
            })
            .collect();
        g.set_rows(GridRows {
            rows,
            total: RowTotal::Exact(10),
            more: false,
        });
        g.scroll.set_viewport(5);
        g
    }

    #[test]
    fn grid_nav_and_keymap() {
        use crate::widgets::grid::{GridAction, GridEvent};
        let mut g = grid();
        g.on_key(&c('j'));
        g.on_key(&c('l'));
        assert_eq!(g.cursor, (1, 1));
        g.on_key(&c('G'));
        assert_eq!(g.cursor, (9, 1), "G is the last row");
        g.on_key(&k(KeyCode::Home));
        assert_eq!(g.cursor, (9, 0), "Home stays in the row");
        let mut g = grid().nav(NavKeys::NO_VIM);
        assert_eq!(g.on_key(&c('j')).0, Outcome::Ignored);
        assert_eq!(g.cursor, (0, 0));
        g.on_key(&k(KeyCode::Down));
        assert_eq!(g.cursor, (1, 0));

        let mut g = grid();
        assert!(matches!(g.on_key(&c('y')).1, Some(GridEvent::Copy(_))));
        let mut g = grid().keymap(vec![(KeyBinding::ctrl('c'), GridAction::Copy)]);
        assert_eq!(
            g.on_key(&c('y')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert_eq!(
            g.on_key(&c('r')).0,
            Outcome::Ignored,
            "every other letter too"
        );
        assert!(matches!(
            g.on_key(&Key {
                code: KeyCode::Char('c'),
                mods: KeyModifiers::CONTROL,
            })
            .1,
            Some(GridEvent::Copy(_))
        ));
        // duplicate_keys rewrites only the DuplicateRow bindings
        let g = grid().duplicate_keys(&[]);
        assert_eq!(binding_for(&g.keymap, GridAction::DuplicateRow), None);
        assert_eq!(
            binding_for(&g.keymap, GridAction::Copy),
            Some(KeyBinding::char('y'))
        );
    }

    #[test]
    fn tabs_nav_and_keymap() {
        use crate::widgets::tabs::{TabAction, TabEvent, TabItem, Tabs};
        let items = || {
            (0..3)
                .map(|i| TabItem::new(&format!("t{i}")).closable())
                .collect()
        };
        let mut t = Tabs::with_items(WidgetId::of("t"), items());
        assert_eq!(
            t.on_key(&c('G')).1,
            Some(TabEvent::Activated(2)),
            "G is End"
        );
        let mut t = Tabs::with_items(WidgetId::of("t"), items()).nav(NavKeys::NO_VIM);
        assert_eq!(t.on_key(&c('l')).0, Outcome::Ignored);
        assert_eq!(t.on_key(&k(KeyCode::Right)).1, Some(TabEvent::Activated(1)));

        let mut t = Tabs::with_items(WidgetId::of("t"), items());
        assert_eq!(t.on_key(&c('x')).1, Some(TabEvent::Close(0)));
        assert_eq!(t.on_key(&c('3')).1, Some(TabEvent::Activated(2)));
        let mut t = Tabs::with_items(WidgetId::of("t"), items())
            .keymap(vec![(KeyBinding::ctrl('w'), TabAction::Close)]);
        assert_eq!(
            t.on_key(&c('x')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert_eq!(t.on_key(&c('3')).0, Outcome::Ignored, "digits too");
        let ctrl_w = Key {
            code: KeyCode::Char('w'),
            mods: KeyModifiers::CONTROL,
        };
        assert_eq!(t.on_key(&ctrl_w).1, Some(TabEvent::Close(0)));
    }

    #[test]
    fn chips_nav_and_keymap() {
        use crate::widgets::chips::{Chip, ChipAction, ChipBar, ChipEvent};
        let bar = || {
            let mut b = ChipBar::new(WidgetId::of("c"));
            b.chips = vec![Chip::new("a"), Chip::new("b")];
            b
        };
        let mut b = bar();
        b.on_key(&c('l'));
        assert_eq!(b.cursor, 1);
        let mut b = bar().nav(NavKeys::NO_VIM);
        assert_eq!(b.on_key(&c('l')).0, Outcome::Ignored);
        b.on_key(&k(KeyCode::End));
        assert_eq!(b.cursor, 2, "End reaches the add button");

        let mut b = bar();
        assert_eq!(b.on_key(&c('x')).1, Some(ChipEvent::Remove(0)));
        let mut b = bar().keymap(vec![(KeyBinding::char('d'), ChipAction::Remove)]);
        assert_eq!(
            b.on_key(&c('x')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert_eq!(b.on_key(&c('+')).0, Outcome::Ignored, "so is add");
        assert_eq!(b.on_key(&c('d')).1, Some(ChipEvent::Remove(0)));
    }

    #[test]
    fn dialog_nav_and_keymap() {
        use crate::core::focus::{Focus, FocusRing};
        use crate::widgets::dialog::{Dialog, DialogAction, DialogResult};
        let ring = FocusRing::default();
        let dialog = || Dialog::confirm(WidgetId::of("q"), "Quit?", "Leave now?", "Quit");
        let mut focus = Focus::default();
        let mut d = dialog();
        focus.focus(d.actions[0].id);
        d.on_key(&c('l'), &mut focus, &ring);
        assert_eq!(
            focus.current(),
            Some(d.actions[1].id),
            "l moves between actions"
        );
        let mut d = dialog().nav(NavKeys::NO_VIM);
        focus.focus(d.actions[0].id);
        d.on_key(&c('l'), &mut focus, &ring);
        assert_eq!(focus.current(), Some(d.actions[0].id));
        d.on_key(&k(KeyCode::Right), &mut focus, &ring);
        assert_eq!(focus.current(), Some(d.actions[1].id));

        let mut d = dialog().keymap(vec![(KeyBinding::char('q'), DialogAction::Confirm)]);
        let mut focus = Focus::default();
        d.on_key(&c('y'), &mut focus, &ring);
        assert_eq!(d.result, None, "the default is unbound");
        d.on_key(&c('n'), &mut focus, &ring);
        assert_eq!(d.result, None, "so is the cancel letter");
        d.on_key(&c('q'), &mut focus, &ring);
        assert!(matches!(d.result, Some(DialogResult::Action(1))));
    }

    #[test]
    fn picker_and_completion_nav() {
        use crate::widgets::completion::{Completion, CompletionItem};
        use crate::widgets::picker::{Picker, PickerItem};
        let items = || {
            (0..5)
                .map(|i| PickerItem::new(format!("p{i}")))
                .collect::<Vec<_>>()
        };
        let mut p = Picker::new(WidgetId::of("p"), "P");
        p.set_items(items());
        p.on_key(&c('j'));
        assert_eq!(p.cursor, 0, "a searchable picker types its letters");
        assert_eq!(p.query, "j");
        p.on_key(&k(KeyCode::Down));
        assert_eq!(p.cursor, 1);
        let mut p = Picker::new(WidgetId::of("p"), "P");
        p.searchable = false;
        p.set_items(items());
        p.on_key(&c('G'));
        assert_eq!(p.cursor, 4, "a plain list takes the whole table");
        let mut p = Picker::new(WidgetId::of("p"), "P").nav(NavKeys::NO_VIM);
        p.searchable = false;
        p.set_items(items());
        p.on_key(&c('j'));
        assert_eq!(p.cursor, 0);

        let item = |l: &str| CompletionItem {
            label: l.to_owned(),
            glyph: "T".into(),
            detail: String::new(),
            insert: l.to_owned(),
            matched: vec![],
        };
        let mut comp = Completion::new(WidgetId::of("c"));
        comp.open(
            vec![item("a"), item("b"), item("c")],
            ratatui::layout::Rect::new(0, 0, 1, 1),
            0,
        );
        assert_eq!(
            comp.on_key(&c('j')).0,
            Outcome::Ignored,
            "letters reach the editor"
        );
        assert_eq!(
            comp.on_key(&k(KeyCode::End)).0,
            Outcome::Ignored,
            "so does End"
        );
        comp.on_key(&k(KeyCode::Down));
        assert_eq!(comp.cursor, 1);
        let mut comp = Completion::new(WidgetId::of("c")).nav(NavKeys::DEFAULT);
        comp.open(
            vec![item("a"), item("b"), item("c")],
            ratatui::layout::Rect::new(0, 0, 1, 1),
            0,
        );
        comp.on_key(&k(KeyCode::End));
        assert_eq!(comp.cursor, 2, "an app can hand it Home/End");
    }

    #[test]
    fn props_and_help_nav() {
        use crate::widgets::help::HelpOverlay;
        use crate::widgets::props::{Prop, PropsList};
        let props = || vec![Prop::new("a", "1"), Prop::new("b", "2")];
        let mut p = PropsList::new(WidgetId::of("p"), props());
        p.on_key(&c('j'));
        assert_eq!(p.cursor, 1);
        let mut p = PropsList::new(WidgetId::of("p"), props()).nav(NavKeys::NO_VIM);
        assert_eq!(p.on_key(&c('j')).0, Outcome::Ignored);
        assert_eq!(p.cursor, 0);

        let rows: Vec<(&str, &str)> = (0..40).map(|_| ("k", "action")).collect();
        let mut h = HelpOverlay::new(WidgetId::of("h"), "S", vec![("Keys", rows.clone())]);
        h.scroll.set_content(40);
        h.scroll.set_viewport(5);
        h.on_key(&c('G'));
        assert!(h.scroll.offset > 0, "G jumps to the end");
        let mut h =
            HelpOverlay::new(WidgetId::of("h"), "S", vec![("Keys", rows)]).nav(NavKeys::NO_VIM);
        h.scroll.set_content(40);
        h.scroll.set_viewport(5);
        h.on_key(&c('j'));
        assert_eq!(h.scroll.offset, 0);
        h.on_key(&k(KeyCode::Down));
        assert_eq!(h.scroll.offset, 1);
    }
}
