use crate::core::event::{ClientPlatform, Key};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavKeys {
    pub arrows: bool,

    pub vim: bool,

    pub page: bool,

    pub home_end: bool,
}

impl NavKeys {
    pub const DESKTOP: Self = Self {
        arrows: true,
        vim: false,
        page: true,
        home_end: true,
    };

    pub const VIM: Self = Self {
        vim: true,
        ..Self::DESKTOP
    };

    pub const DEFAULT: Self = Self::DESKTOP;

    pub const NO_VIM: Self = Self::DESKTOP;

    pub const fn vim(mut self, on: bool) -> Self {
        self.vim = on;
        self
    }

    pub const fn page(mut self, on: bool) -> Self {
        self.page = on;
        self
    }

    pub const fn home_end(mut self, on: bool) -> Self {
        self.home_end = on;
        self
    }

    pub const fn arrows(mut self, on: bool) -> Self {
        self.arrows = on;
        self
    }

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
            KeyCode::Char(c) if self.vim && !key.chord() => match c {
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

    pub const fn move_hint(&self, type_ahead: bool) -> Option<&'static str> {
        if self.arrows {
            Some("↑↓")
        } else if self.vim && !type_ahead {
            Some("j k")
        } else {
            None
        }
    }

    pub const fn page_hint(&self) -> Option<&'static str> {
        if self.page { Some("PgUp PgDn") } else { None }
    }

    pub const fn ends_hint(&self, type_ahead: bool) -> Option<&'static str> {
        if self.vim && !type_ahead {
            Some("g G")
        } else if self.home_end {
            Some("Home End")
        } else {
            None
        }
    }

    pub fn is_vim_letter(key: &Key) -> bool {
        matches!(
            key.code,
            KeyCode::Char('k' | 'K' | 'j' | 'J' | 'h' | 'H' | 'l' | 'L' | 'g' | 'G')
        )
    }
}

impl Default for NavKeys {
    fn default() -> Self {
        crate::core::list::NavProfile::current().nav()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub mods: KeyModifiers,

    pub alt_stand_in: bool,
}

impl KeyBinding {
    pub const fn new(code: KeyCode, mods: KeyModifiers) -> Self {
        Self {
            code,
            mods,
            alt_stand_in: false,
        }
    }

    pub const fn alt_stand_in(mut self) -> Self {
        self.alt_stand_in = true;
        self
    }

    pub const fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub const fn char(c: char) -> Self {
        Self::key(KeyCode::Char(c))
    }

    pub const fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    pub const fn alt(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    pub const fn cmd(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::SUPER)
    }

    pub const fn shift(self) -> Self {
        Self::new(self.code, self.mods.union(KeyModifiers::SHIFT))
    }

    pub fn deliverable(&self) -> bool {
        use crate::core::keyboard::{alt_delivered, super_delivered};
        (!self.mods.contains(KeyModifiers::SUPER) || super_delivered())
            && (!self.mods.contains(KeyModifiers::ALT) || alt_delivered())
            && (!self.alt_stand_in || !alt_delivered())
    }

    pub fn matches(&self, key: &Key) -> bool {
        if self.code != key.code {
            return false;
        }
        if self.alt_stand_in && crate::core::keyboard::alt_delivered() {
            return false;
        }
        if matches!(self.code, KeyCode::Char(_)) {
            (!self.mods.contains(KeyModifiers::SHIFT) || key.shift())
                && self.mods.difference(KeyModifiers::SHIFT)
                    == key.mods.difference(KeyModifiers::SHIFT)
        } else {
            self.mods == key.mods
        }
    }

    pub fn label(&self) -> String {
        self.label_on(ClientPlatform::current())
    }

    pub fn label_on(&self, platform: ClientPlatform) -> String {
        let mut out = String::new();
        let char_shift =
            self.mods.contains(KeyModifiers::SHIFT) && matches!(self.code, KeyCode::Char(_));
        if self.mods.contains(KeyModifiers::SUPER) {
            if platform.is_macos() {
                out.push_str(if self.mods.contains(KeyModifiers::SHIFT) {
                    "⇧⌘"
                } else {
                    "⌘"
                });
            } else {
                out.push_str("Super+");
            }
        }
        if self.mods.contains(KeyModifiers::CONTROL) {
            out.push_str("Ctrl+");
        }
        if self.mods.contains(KeyModifiers::ALT) {
            out.push_str("Alt+");
        }
        let ctrl_or_alt = self
            .mods
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER);
        let mac_cmd = self.mods.contains(KeyModifiers::SUPER) && platform.is_macos();

        let back_tab = self.code == KeyCode::BackTab;
        if self.mods.contains(KeyModifiers::SHIFT)
            && !mac_cmd
            && !back_tab
            && (!char_shift || ctrl_or_alt)
        {
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

pub type KeyMap<A> = Vec<(KeyBinding, A)>;

pub trait Actions: Copy + PartialEq + Sized + 'static {
    fn default_keymap() -> KeyMap<Self>;

    fn label(self) -> &'static str;
}

pub fn lookup<A: Copy>(map: &[(KeyBinding, A)], key: &Key) -> Option<A> {
    map.iter().find(|(b, _)| b.matches(key)).map(|&(_, a)| a)
}

pub fn unassigned_chord<A: Copy>(key: &Key, map: &[(KeyBinding, A)]) -> bool {
    matches!(key.code, KeyCode::Char(_)) && key.chord() && lookup(map, key).is_none()
}

pub fn binding_for<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<KeyBinding> {
    map.iter().find(|(_, a)| *a == action).map(|&(b, _)| b)
}

pub fn deliverable_binding_for<A: PartialEq>(
    map: &[(KeyBinding, A)],
    action: A,
) -> Option<KeyBinding> {
    map.iter()
        .find(|(b, a)| *a == action && b.deliverable())
        .map(|&(b, _)| b)
}

pub fn label_for<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<String> {
    deliverable_binding_for(map, action).map(|b| b.label())
}

pub const SELECT_ALL: [KeyBinding; 2] = [KeyBinding::cmd('a'), KeyBinding::ctrl('a')];

pub fn select_all_bindings<A: Copy>(action: A) -> impl Iterator<Item = (KeyBinding, A)> {
    SELECT_ALL.into_iter().map(move |b| (b, action))
}

pub fn hint_key<A: PartialEq>(map: &[(KeyBinding, A)], action: A) -> Option<&'static str> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static LABELS: OnceLock<Mutex<HashMap<KeyBinding, &'static str>>> = OnceLock::new();
    let b = deliverable_binding_for(map, action)?;
    let mut labels = LABELS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Some(labels.entry(b).or_insert_with(|| intern(b.label())))
}

pub fn intern(label: String) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static LABELS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut labels = LABELS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(&l) = labels.get(label.as_str()) {
        return l;
    }
    let l: &'static str = Box::leak(label.into_boxed_str());
    labels.insert(l);
    l
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
    fn nav_hints_name_only_keys_that_navigate() {
        use crate::core::list::KeyStep;

        let navigates = |n: &NavKeys, ta: bool, code: KeyCode| {
            matches!(
                KeyStep::of(&key(code), n, ta),
                Some(KeyStep::Nav(Nav::Home))
            )
        };
        let only_vim = NavKeys::VIM.home_end(false).arrows(false).page(false);
        for n in [
            NavKeys::DESKTOP,
            NavKeys::VIM,
            NavKeys::VIM.home_end(false),
            only_vim,
        ] {
            for ta in [false, true] {
                let ok = match n.ends_hint(ta) {
                    Some("Home End") => navigates(&n, ta, KeyCode::Home),
                    Some("g G") => navigates(&n, ta, KeyCode::Char('g')),
                    Some(other) => panic!("{other}"),
                    None => {
                        !navigates(&n, ta, KeyCode::Home) && !navigates(&n, ta, KeyCode::Char('g'))
                    }
                };
                assert!(ok, "{n:?} type-ahead {ta}");
            }
        }
        assert_eq!(NavKeys::DESKTOP.ends_hint(true), Some("Home End"));
        assert_eq!(NavKeys::VIM.ends_hint(false), Some("g G"));
        assert_eq!(NavKeys::VIM.ends_hint(true), Some("Home End"));
        assert_eq!(only_vim.ends_hint(true), None);
        assert_eq!(only_vim.move_hint(false), Some("j k"));
        assert_eq!(only_vim.move_hint(true), None);
        assert_eq!(only_vim.page_hint(), None);
        assert_eq!(NavKeys::DESKTOP.move_hint(true), Some("↑↓"));
    }

    #[test]
    fn the_shared_nav_table() {
        let d = NavKeys::VIM;
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
        let d = NavKeys::VIM;
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
    fn hints_name_cmd_only_once_the_terminal_delivers_it() {
        let map: KeyMap<TestCmd> = select_all_bindings(TestCmd::Add).collect();
        assert_eq!(label_for(&map, TestCmd::Add).as_deref(), Some("Ctrl+A"));
        assert_eq!(hint_key(&map, TestCmd::Add), Some("Ctrl+A"));
        crate::core::keyboard::set_super_delivered(true);
        let cmd = KeyBinding::cmd('a').label();
        assert_eq!(label_for(&map, TestCmd::Add), Some(cmd.clone()));
        assert_eq!(hint_key(&map, TestCmd::Add), Some(cmd.as_str()));
        assert_eq!(
            lookup(&map, &with(KeyCode::Char('a'), KeyModifiers::SUPER)),
            Some(TestCmd::Add)
        );
        assert_eq!(
            lookup(&map, &with(KeyCode::Char('a'), KeyModifiers::CONTROL)),
            Some(TestCmd::Add)
        );
    }

    #[test]
    fn alt_chords_are_hinted_only_where_alt_arrives_and_stand_ins_only_where_not() {
        use crate::core::keyboard::{AltKeys, set_alt_keys};
        let map: KeyMap<TestCmd> = vec![
            (KeyBinding::alt('z'), TestCmd::Add),
            (
                KeyBinding::new(KeyCode::Right, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
                    .alt_stand_in(),
                TestCmd::Add,
            ),
        ];
        let stand_in = with(KeyCode::Right, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        let alt_z = with(KeyCode::Char('z'), KeyModifiers::ALT);
        set_alt_keys(AltKeys::Delivered);
        assert_eq!(hint_key(&map, TestCmd::Add), Some("Alt+Z"));
        assert_eq!(lookup(&map, &alt_z), Some(TestCmd::Add));
        assert_eq!(
            lookup(&map, &stand_in),
            None,
            "Ctrl+Shift+→ stays the content's"
        );

        set_alt_keys(AltKeys::UntilSeen);
        assert_eq!(hint_key(&map, TestCmd::Add), Some("Ctrl+Shift+→"));
        assert_eq!(lookup(&map, &stand_in), Some(TestCmd::Add));
        assert_eq!(lookup(&map, &alt_z), Some(TestCmd::Add), "still bound");
    }

    #[test]
    fn cmd_letters_never_navigate() {
        let d = NavKeys::DEFAULT;
        assert_eq!(
            d.action(&with(KeyCode::Char('j'), KeyModifiers::SUPER)),
            None
        );
    }

    #[test]
    fn each_group_switches_off_on_its_own() {
        let no_vim = NavKeys::NO_VIM;
        for c in ['h', 'j', 'k', 'l', 'g', 'G', 'J', 'K'] {
            assert_eq!(no_vim.action(&key(KeyCode::Char(c))), None, "{c}");
        }
        assert_eq!(no_vim.action(&key(KeyCode::Down)), Some(Nav::Down));
        let no_page = NavKeys::VIM.page(false);
        assert_eq!(no_page.action(&key(KeyCode::PageDown)), None);
        assert_eq!(no_page.action(&key(KeyCode::Char('j'))), Some(Nav::Down));
        let no_home = NavKeys::VIM.home_end(false);
        assert_eq!(no_home.action(&key(KeyCode::Home)), None);
        assert_eq!(no_home.action(&key(KeyCode::Char('g'))), Some(Nav::Home));
        let no_arrows = NavKeys::VIM.arrows(false);
        assert_eq!(no_arrows.action(&key(KeyCode::Up)), None);
        assert_eq!(NavKeys::default(), NavKeys::DEFAULT);
        assert_eq!(
            NavKeys::DEFAULT,
            NavKeys::DESKTOP,
            "desktop keys by default (K33)"
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum TestCmd {
        Remove,
        Add,
    }

    impl Actions for TestCmd {
        fn label(self) -> &'static str {
            "test"
        }
        fn default_keymap() -> KeyMap<Self> {
            vec![
                (KeyBinding::char('x'), TestCmd::Remove),
                (KeyBinding::key(KeyCode::Delete), TestCmd::Remove),
                (KeyBinding::char('+'), TestCmd::Add),
            ]
        }
    }

    #[test]
    fn bindings_match_lookup_and_label() {
        let map = TestCmd::default_keymap();
        assert_eq!(
            lookup(&map, &key(KeyCode::Char('x'))),
            Some(TestCmd::Remove)
        );
        assert_eq!(lookup(&map, &key(KeyCode::Delete)), Some(TestCmd::Remove));
        assert_eq!(
            lookup(&map, &with(KeyCode::Char('+'), KeyModifiers::SHIFT)),
            Some(TestCmd::Add),
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
        assert_eq!(label_for(&map, TestCmd::Remove).as_deref(), Some("x"));
        assert_eq!(KeyBinding::ctrl('s').label(), "Ctrl+S");
        assert_eq!(KeyBinding::key(KeyCode::Delete).label(), "Delete");
        assert_eq!(hint_key(&map, TestCmd::Add), Some("+"));
        assert_eq!(
            hint_key(&map, TestCmd::Add).map(str::as_ptr),
            hint_key(&map, TestCmd::Add).map(str::as_ptr),
            "labels are interned once"
        );
        assert_eq!(KeyBinding::char(' ').label(), "Space");
        let cmd_a = KeyBinding::cmd('a');
        assert_eq!(cmd_a.label_on(ClientPlatform::MacOs), "⌘A");
        assert_eq!(cmd_a.label_on(ClientPlatform::Other), "Super+A");
        assert!(cmd_a.matches(&with(KeyCode::Char('a'), KeyModifiers::SUPER)));
        assert!(!cmd_a.matches(&with(KeyCode::Char('a'), KeyModifiers::CONTROL)));
        let remapped: KeyMap<TestCmd> = vec![(KeyBinding::char('d'), TestCmd::Remove)];
        assert_eq!(lookup(&remapped, &key(KeyCode::Char('x'))), None);
        assert_eq!(binding_for(&remapped, TestCmd::Add), None);
    }
}

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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::list::{ListBox, ListCmd, ListItem, SelectMode};
        let items = || vec![ListItem::new("a"), ListItem::new("b"), ListItem::new("c")];
        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Single);
        l.on_key(&c('j'));
        assert_eq!(l.cursor, 1, "vim on under the Vim profile");
        let mut l =
            ListBox::new(WidgetId::of("l"), items(), SelectMode::Single).nav(NavKeys::NO_VIM);
        assert_eq!(l.on_key(&c('j')), Outcome::Ignored);
        assert_eq!(l.cursor, 0);
        l.on_key(&k(KeyCode::Down));
        assert_eq!(l.cursor, 1, "arrows stay");

        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Multi);
        l.on_key(&c('a'));
        assert_eq!(
            l.checked_count(),
            l.items.len(),
            "`a` checks all by default"
        );
        let mut l = ListBox::new(WidgetId::of("l"), items(), SelectMode::Multi)
            .keymap(vec![(KeyBinding::ctrl('a'), ListCmd::ToggleAll)]);
        assert_eq!(
            l.on_key(&c('a')),
            Outcome::Ignored,
            "the default is unbound"
        );
        l.on_key(&Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::CONTROL,
        });
        assert_eq!(l.checked_count(), l.items.len(), "the remapped key fires");
    }

    #[test]
    fn tree_nav_and_keymap() {
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::tree::{TreeCmd, TreeNode, TreeView};
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
            .keymap(vec![(KeyBinding::char('e'), TreeCmd::ExpandAll)]);
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::table::{Cell, Column, DataTable, TableCmd};
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
        let mut t = table().keymap(vec![(KeyBinding::char('o'), TableCmd::SortByColumn)]);
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::select::Select;
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0);
        s.on_key(&k(KeyCode::Enter));
        s.on_key(&c('G'));
        assert_eq!(
            s.list.cursor.index, 2,
            "the open list takes the whole table (G = End)"
        );
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0).nav(NavKeys::NO_VIM);
        s.on_key(&k(KeyCode::Enter));
        s.on_key(&c('j'));
        assert_eq!(s.list.cursor.index, 0);
        s.on_key(&k(KeyCode::Down));
        assert_eq!(s.list.cursor.index, 1);
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0);
        assert_eq!(
            s.on_key(&c('j')).0,
            Outcome::Ignored,
            "a closed field leaves letters to the page"
        );
    }

    #[test]
    fn menus_nav() {
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::menu::{ContextMenu, MenuBar, MenuItem};
        let menu = || {
            (0..20)
                .map(|i| MenuItem::new(format!("item {i}")))
                .collect::<Vec<_>>()
        };
        let mut m = ContextMenu::new(WidgetId::of("m"), menu());
        m.on_key(&c('j'));
        assert_eq!(m.list.cursor.index, 1);
        let mut m = ContextMenu::new(WidgetId::of("m"), menu()).nav(NavKeys::NO_VIM);
        m.on_key(&c('j'));
        assert_eq!(m.list.cursor.index, 0);
        m.on_key(&k(KeyCode::Down));
        assert_eq!(m.list.cursor.index, 1);

        let mut bar = MenuBar::new(WidgetId::of("bar"), vec![("A", menu()), ("B", menu())])
            .nav(NavKeys::NO_VIM);
        bar.on_key(&c('l'));
        assert_eq!(bar.cursor, 0);
        bar.on_key(&k(KeyCode::Right));
        assert_eq!(bar.cursor, 1);
        bar.on_key(&k(KeyCode::Enter));
        let open = bar.open.as_ref().expect("menu opens");
        assert_eq!(
            open.list.nav,
            NavKeys::NO_VIM,
            "the open menu inherits the bar's keys"
        );
    }

    #[test]
    fn radio_and_steps_nav() {
        crate::core::list::NavProfile::Vim.install();
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::panel::{PanelCmd, ScrollPanel};
        use crate::widgets::textarea::TextArea;
        let mut t = TextArea::new(WidgetId::of("t"), "T", 3)
            .edit_entry(crate::core::edit::EditEntry::Explicit);
        assert_ne!(t.on_key(&c('j')).0, Outcome::Ignored);
        let mut t = TextArea::new(WidgetId::of("t"), "T", 3)
            .edit_entry(crate::core::edit::EditEntry::Explicit)
            .nav(NavKeys::NO_VIM);
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
            .keymap(vec![(KeyBinding::char('t'), PanelCmd::Follow)]);
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::viewport::{TextViewport, ViewportCmd};
        let mut v = TextViewport::new(WidgetId::of("v"));
        assert_ne!(v.on_key(&c('k')).0, Outcome::Ignored);
        let mut v = TextViewport::new(WidgetId::of("v")).nav(NavKeys::NO_VIM);
        assert_eq!(v.on_key(&c('k')).0, Outcome::Ignored);

        let mut v = TextViewport::new(WidgetId::of("v"));
        assert!(v.follow);
        v.on_key(&c('f'));
        assert!(!v.follow, "`f` toggles following by default");
        let mut v = TextViewport::new(WidgetId::of("v"))
            .keymap(vec![(KeyBinding::char('p'), ViewportCmd::ToggleFollow)]);
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::code::{CodeCmd, CodeEditor};
        let text = "a\nb\nc\n";
        let mut e = CodeEditor::new(WidgetId::of("e"), text)
            .edit_entry(crate::core::edit::EditEntry::Explicit);
        e.on_key(&c('j'));
        assert_eq!(e.field.buffer.cursor_pos().line, 1);
        let mut e = CodeEditor::new(WidgetId::of("e"), text)
            .edit_entry(crate::core::edit::EditEntry::Explicit)
            .nav(NavKeys::NO_VIM);
        assert_eq!(e.on_key(&c('j')).0, Outcome::Ignored);
        assert_eq!(e.field.buffer.cursor_pos().line, 0);

        let mut e = CodeEditor::new(WidgetId::of("e"), text)
            .edit_entry(crate::core::edit::EditEntry::Explicit);
        e.on_key(&c('/'));
        assert!(e.find.is_some(), "`/` opens find by default");
        let mut e = CodeEditor::new(WidgetId::of("e"), text)
            .edit_entry(crate::core::edit::EditEntry::Explicit)
            .keymap(vec![(KeyBinding::ctrl('f'), CodeCmd::Find)]);
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::grid::{GridAction, GridCmd};
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
        assert!(matches!(g.on_key(&c('y')).1, Some(GridAction::Copy(_))));
        let mut g = grid().keymap(vec![(KeyBinding::ctrl('c'), GridCmd::Copy)]);
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
            Some(GridAction::Copy(_))
        ));

        let g = grid().duplicate_keys(&[]);
        assert_eq!(binding_for(&g.keymap, GridCmd::DuplicateRow), None);
        assert_eq!(
            binding_for(&g.keymap, GridCmd::Copy),
            Some(KeyBinding::cmd('c'))
        );
        assert_eq!(
            label_for(&g.keymap, GridCmd::Copy).as_deref(),
            Some("y"),
            "Cmd+C is not named until the terminal delivers Cmd"
        );
    }

    #[test]
    fn tabs_nav_and_keymap() {
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::tabs::{TabAction, TabCmd, TabItem, Tabs};
        let items = || {
            (0..3)
                .map(|i| TabItem::new(&format!("t{i}")).closable())
                .collect()
        };
        let mut t = Tabs::with_items(WidgetId::of("t"), items());
        assert_eq!(
            t.on_key(&c('G')).1,
            Some(TabAction::Activated(2)),
            "G is End"
        );
        let mut t = Tabs::with_items(WidgetId::of("t"), items()).nav(NavKeys::NO_VIM);
        assert_eq!(t.on_key(&c('l')).0, Outcome::Ignored);
        assert_eq!(
            t.on_key(&k(KeyCode::Right)).1,
            Some(TabAction::Activated(1))
        );

        let mut t = Tabs::with_items(WidgetId::of("t"), items());
        assert_eq!(t.on_key(&c('x')).1, Some(TabAction::Close(0)));
        assert_eq!(t.on_key(&c('3')).1, Some(TabAction::Activated(2)));
        let mut t = Tabs::with_items(WidgetId::of("t"), items())
            .keymap(vec![(KeyBinding::ctrl('w'), TabCmd::Close)]);
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
        assert_eq!(t.on_key(&ctrl_w).1, Some(TabAction::Close(0)));
    }

    #[test]
    fn chips_nav_and_keymap() {
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::chips::{Chip, ChipAction, ChipBar, ChipCmd};
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
        assert_eq!(b.on_key(&c('x')).1, Some(ChipAction::Remove(0)));
        let mut b = bar().keymap(vec![(KeyBinding::char('d'), ChipCmd::Remove)]);
        assert_eq!(
            b.on_key(&c('x')).0,
            Outcome::Ignored,
            "the default is unbound"
        );
        assert_eq!(b.on_key(&c('+')).0, Outcome::Ignored, "so is add");
        assert_eq!(b.on_key(&c('d')).1, Some(ChipAction::Remove(0)));
    }

    #[test]
    fn dialog_nav_and_keymap() {
        crate::core::list::NavProfile::Vim.install();
        use crate::core::focus::{Focus, FocusRing};
        use crate::widgets::dialog::{Dialog, DialogCmd, DialogResult};
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

        let mut d = dialog().keymap(vec![(KeyBinding::char('q'), DialogCmd::Confirm)]);
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
        crate::core::list::NavProfile::Vim.install();
        use crate::widgets::completion::Completion;
        use crate::widgets::option_list::Item;
        use crate::widgets::picker::Picker;
        let items = || {
            (0..5)
                .map(|i| Item::new(format!("p{i}")))
                .collect::<Vec<_>>()
        };
        let mut p = Picker::new(WidgetId::of("p"), "P");
        p.set_items(items());
        p.on_key(&c('j'));
        assert_eq!(
            p.list.cursor.index, 0,
            "a searchable picker types its letters"
        );
        assert_eq!(p.query(), "j");
        p.on_key(&k(KeyCode::Down));
        assert_eq!(p.list.cursor.index, 1);
        let mut p = Picker::new(WidgetId::of("p"), "P");
        p.set_searchable(false);
        p.set_items(items());
        p.on_key(&c('G'));
        assert_eq!(p.list.cursor.index, 4, "a plain list takes the whole table");
        let mut p = Picker::new(WidgetId::of("p"), "P").nav(NavKeys::NO_VIM);
        p.set_searchable(false);
        p.set_items(items());
        p.on_key(&c('j'));
        assert_eq!(p.list.cursor.index, 0);

        let item = |l: &str| Item::new(l).glyph("T");
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
        assert_eq!(comp.list.cursor.index, 1);
        let mut comp = Completion::new(WidgetId::of("c")).nav(NavKeys::DEFAULT);
        comp.open(
            vec![item("a"), item("b"), item("c")],
            ratatui::layout::Rect::new(0, 0, 1, 1),
            0,
        );
        comp.on_key(&k(KeyCode::End));
        assert_eq!(comp.list.cursor.index, 2, "an app can hand it Home/End");
    }

    #[test]
    fn props_and_help_nav() {
        crate::core::list::NavProfile::Vim.install();
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
