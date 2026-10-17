//! The app's chrome (plan §1.8, `Chrome`): the rows around the page and
//! the menus whose rows carry typed commands.
//!
//! - [`Chrome`] names the frame's regions once (header rows, the gap to
//!   the body, the footer rows, the body's side inset), so no app does
//!   its own header/body/footer `Rect` arithmetic (A17), and a dialog is
//!   placed clear of exactly those rows ([`Chrome::fit`]).
//! - [`Menus`] builds a [`MenuBar`] (or a context menu) whose items each
//!   carry the app's command `C`; a choice resolves to that command
//!   ([`Menus::command`]), never to the item's label (no label-string
//!   re-dispatch, M23).

use ratatui::layout::Rect;

use crate::core::id::WidgetId;
use crate::widgets::dialog::Dialog;
use crate::widgets::menu::{MenuBar, MenuBarAction, MenuItem};

/// The regions of one frame ([`Chrome::regions`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Regions {
    /// The header rows (a brand, a menu bar, a trail); empty with none.
    pub header: Rect,
    /// Where the page draws.
    pub body: Rect,
    /// The footer rows (hints and status); empty with none.
    pub footer: Rect,
}

/// How an app frames its pages: rows above and below the body and the gap
/// between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    /// Rows of header at the top. Default 1.
    pub header: u16,
    /// Blank rows between the header and the body, and between the body
    /// and the footer. Default 1.
    pub gap: u16,
    /// Rows of footer at the bottom. Default 1.
    pub footer: u16,
    /// Columns the body keeps clear on each side. Default 0.
    pub inset: u16,
}

impl Default for Chrome {
    fn default() -> Self {
        Self::new()
    }
}

impl Chrome {
    /// One header row, one footer row, a blank row between each and the
    /// body, no side inset.
    pub const fn new() -> Self {
        Self {
            header: 1,
            gap: 1,
            footer: 1,
            inset: 0,
        }
    }

    /// See [`Self::header`].
    pub const fn header(mut self, rows: u16) -> Self {
        self.header = rows;
        self
    }

    /// See [`Self::gap`].
    pub const fn gap(mut self, rows: u16) -> Self {
        self.gap = rows;
        self
    }

    /// See [`Self::footer`].
    pub const fn footer(mut self, rows: u16) -> Self {
        self.footer = rows;
        self
    }

    /// See [`Self::inset`].
    pub const fn inset(mut self, cols: u16) -> Self {
        self.inset = cols;
        self
    }

    /// The header, body and footer of `area`. On a screen too short for
    /// them all the body shrinks first (to nothing), then the footer
    /// keeps the last row and the header the first.
    pub fn regions(&self, area: Rect) -> Regions {
        let header_h = self.header.min(area.height);
        let footer_h = self.footer.min(area.height - header_h);
        let header = Rect::new(area.x, area.y, area.width, header_h);
        let footer = Rect::new(area.x, area.bottom() - footer_h, area.width, footer_h);
        let top = if header_h > 0 { header_h + self.gap } else { 0 };
        let bottom = if footer_h > 0 { footer_h + self.gap } else { 0 };
        let body_y = (area.y + top).min(area.bottom());
        let body_h = area.height.saturating_sub(top + bottom);
        let body = Rect::new(
            area.x + self.inset.min(area.width / 2),
            body_y,
            area.width.saturating_sub(self.inset * 2),
            body_h,
        );
        Regions {
            header,
            body,
            footer,
        }
    }

    /// The rows a modal keeps clear of: the header and the footer
    /// (a dialog's own margin then keeps it off the gap).
    pub fn rows(&self) -> (u16, u16) {
        (self.header, self.footer)
    }

    /// Place `d` clear of the chrome rows, keeping any larger
    /// [`Dialog::chrome`] it asked for.
    pub fn fit(&self, d: &mut Dialog) {
        let (top, bottom) = self.rows();
        d.chrome = (d.chrome.0.max(top), d.chrome.1.max(bottom));
    }
}

/// Menus whose items carry the app's command `C` (a menu bar's titles,
/// or one context menu): build the widget from them, and resolve a
/// choice to its command.
#[derive(Debug, Clone)]
pub struct Menus<C> {
    menus: Vec<(String, Vec<(MenuItem, C)>)>,
}

impl<C> Default for Menus<C> {
    fn default() -> Self {
        Self { menus: vec![] }
    }
}

impl<C: Clone> Menus<C> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a menu titled `title`, each item with its command.
    pub fn menu(mut self, title: impl Into<String>, items: Vec<(MenuItem, C)>) -> Self {
        self.menus.push((title.into(), items));
        self
    }

    /// The menu titles, in order.
    pub fn titles(&self) -> impl Iterator<Item = &str> {
        self.menus.iter().map(|(t, _)| t.as_str())
    }

    /// The items of menu `m`, without their commands.
    pub fn items(&self, m: usize) -> Vec<MenuItem> {
        self.menus
            .get(m)
            .map(|(_, v)| v.iter().map(|(i, _)| i.clone()).collect())
            .unwrap_or_default()
    }

    /// A menu bar `id` showing these menus.
    pub fn bar(&self, id: WidgetId) -> MenuBar {
        MenuBar::new(
            id,
            self.menus
                .iter()
                .enumerate()
                .map(|(m, (t, _))| (t.as_str(), self.items(m)))
                .collect(),
        )
    }

    /// The command of item `i` of menu `m`.
    pub fn get(&self, m: usize, i: usize) -> Option<C> {
        self.menus.get(m)?.1.get(i).map(|(_, c)| c.clone())
    }

    /// The command a menu bar's `action` chose, if it chose one (a
    /// disabled item is never chosen by the bar).
    pub fn command(&self, action: MenuBarAction) -> Option<C> {
        match action {
            MenuBarAction::Chosen(m, i) => self.get(m, i),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_split_the_frame_once() {
        let area = Rect::new(0, 0, 80, 24);
        let r = Chrome::new().regions(area);
        assert_eq!(r.header, Rect::new(0, 0, 80, 1));
        assert_eq!(r.footer, Rect::new(0, 23, 80, 1));
        assert_eq!(r.body, Rect::new(0, 2, 80, 20));
        let r = Chrome::new().inset(1).regions(area);
        assert_eq!(r.body, Rect::new(1, 2, 78, 20));
        let r = Chrome::new().header(0).gap(0).regions(area);
        assert_eq!(r.body, Rect::new(0, 0, 80, 23));
        assert!(r.header.is_empty());
        // a tiny screen keeps its rows without panicking
        let r = Chrome::new().regions(Rect::new(0, 0, 10, 2));
        assert_eq!((r.header.height, r.footer.height, r.body.height), (1, 1, 0));
        let r = Chrome::new().regions(Rect::new(0, 0, 10, 0));
        assert!(r.body.is_empty() && r.footer.is_empty());
    }

    #[test]
    fn a_dialog_keeps_clear_of_the_chrome_rows() {
        let mut d = Dialog::confirm(WidgetId::of("d"), "Quit?", "Leave now.", "Quit");
        Chrome::new().fit(&mut d);
        assert_eq!(d.chrome, (1, 1));
        let mut d = d.chrome(3, 0);
        Chrome::new().fit(&mut d);
        assert_eq!(d.chrome, (3, 1), "a larger request stays");
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Cmd {
        Open,
        Quit,
        About,
    }

    #[test]
    fn a_menu_choice_resolves_to_its_command_not_its_label() {
        let menus = Menus::new()
            .menu(
                "File",
                vec![
                    (MenuItem::new("Open…"), Cmd::Open),
                    (MenuItem::new("Quit"), Cmd::Quit),
                ],
            )
            .menu("Help", vec![(MenuItem::new("Quit"), Cmd::About)]);
        let bar = menus.bar(WidgetId::of("bar"));
        assert_eq!(bar.labels, ["File", "Help"]);
        assert_eq!(menus.command(MenuBarAction::Chosen(0, 1)), Some(Cmd::Quit));
        // the same label in another menu is another command
        assert_eq!(menus.command(MenuBarAction::Chosen(1, 0)), Some(Cmd::About));
        assert_eq!(menus.command(MenuBarAction::Opened(0)), None);
        assert_eq!(menus.command(MenuBarAction::Chosen(4, 0)), None);
        assert_eq!(menus.titles().collect::<Vec<_>>(), ["File", "Help"]);
    }
}
