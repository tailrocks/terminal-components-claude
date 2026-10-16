//! A keyboard-shortcut reference overlay: titled sections of key/action
//! rows laid out in as many columns as fit, scrollable, in the shared
//! modal [`Frame`](crate::ui::modal::Frame).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::scroll::ScrollState;
use crate::ui::ctx::RenderCtx;
use crate::ui::modal::Frame;
use crate::ui::popup::Placement;
use crate::ui::text::{truncate, width};
use crate::widgets::button::Button;
use crate::widgets::scrollbar;

/// A [`HelpOverlay`] action a key can be bound to. Esc (per the
/// [`Dismiss`](crate::ui::popup::Dismiss) policy) and Enter always close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpAction {
    /// Close the overlay. Default `?` and `q`.
    Close,
}

impl Actions for HelpAction {
    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::char('?'), HelpAction::Close),
            (KeyBinding::char('q'), HelpAction::Close),
        ]
    }
}

pub struct HelpOverlay {
    /// Which dismissals close it. Default `Dismiss::LIGHT`.
    pub dismiss: crate::ui::popup::Dismiss,
    pub id: WidgetId,
    pub scope: String,
    pub sections: Vec<(String, Vec<(String, String)>)>,
    pub scroll: ScrollState,
    pub area: Rect,
    pub closed: bool,
    /// Frame title. Default "Keyboard shortcuts".
    pub title: String,
    /// The closing hint row. Default "↑↓ Scroll · Esc Close".
    pub hint: String,
    /// Which keys scroll (default [`NavKeys::DEFAULT`]).
    pub nav: NavKeys,
    /// The overlay's action keys (default [`HelpAction::default_keymap`]).
    pub keymap: KeyMap<HelpAction>,
    /// Largest share of the screen the overlay takes, in percent of its
    /// width and height. Default 80, so the dimmed page stays visible
    /// around it and a click there closes it (under `Dismiss::LIGHT`).
    pub max_percent: u16,
    /// The button that closes the overlay by pointer. Its label follows
    /// [`crate::theme::Strings::done`].
    pub done: Button,
    /// The theme's strings as of the last render: a title and hint still
    /// reading them follow a change ([`crate::theme::Strings::follow`]).
    strings: crate::theme::Strings,
}

impl HelpOverlay {
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<HelpAction>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn new(id: WidgetId, scope: &str, sections: Vec<(&str, Vec<(&str, &str)>)>) -> Self {
        Self {
            dismiss: crate::ui::popup::Dismiss::LIGHT,
            id,
            scope: scope.to_owned(),
            sections: sections
                .into_iter()
                .map(|(t, rows)| {
                    (
                        t.to_owned(),
                        rows.into_iter()
                            .map(|(k, a)| (k.to_owned(), a.to_owned()))
                            .collect(),
                    )
                })
                .collect(),
            scroll: ScrollState::default(),
            area: Rect::ZERO,
            closed: false,
            title: crate::theme::Strings::DEFAULT.help_title.into(),
            hint: crate::theme::Strings::DEFAULT.help_footer.into(),
            nav: NavKeys::DEFAULT,
            keymap: HelpAction::default_keymap(),
            max_percent: 80,
            done: Button::secondary(id.sub("done"), crate::theme::Strings::DEFAULT.done),
            strings: crate::theme::Strings::DEFAULT,
        }
    }

    /// See [`Self::max_percent`] (clamped to 20..=100).
    pub fn max_percent(mut self, percent: u16) -> Self {
        self.max_percent = percent.clamp(20, 100);
        self
    }

    /// A completed click inside the overlay: the Done button and the
    /// frame's close button close it.
    pub fn on_click(&mut self, id: WidgetId) -> Outcome {
        if id == self.done.id && self.done.on_click()
            || id == crate::ui::modal::CLOSE && self.dismiss.shows_close()
        {
            self.closed = true;
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    pub fn dismiss(mut self, dismiss: crate::ui::popup::Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if let Some(nav) = self.nav.action(key) {
            match nav {
                Nav::Down => self.scroll.scroll_by(1),
                Nav::Up => self.scroll.scroll_by(-1),
                Nav::PageDown => self.scroll.page_down(),
                Nav::PageUp => self.scroll.page_up(),
                Nav::Home => self.scroll.jump_start(),
                Nav::End => self.scroll.jump_end(),
                Nav::Left | Nav::Right => return Outcome::Consumed,
            };
            return Outcome::Changed;
        }
        if let Some(HelpAction::Close) = lookup(&self.keymap, key) {
            self.closed = true;
            return Outcome::Changed;
        }
        match key.code {
            KeyCode::Esc if !self.dismiss.escape => Outcome::Consumed,
            KeyCode::Esc | KeyCode::Enter => {
                self.closed = true;
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let s = ctx.theme.strings;
        let old = std::mem::replace(&mut self.strings, s);
        crate::theme::Strings::follow(&mut self.title, old.help_title, s.help_title);
        crate::theme::Strings::follow(&mut self.hint, old.help_footer, s.help_footer);
        crate::theme::Strings::follow(&mut self.done.label, old.done, s.done);
        let pct = |n: u16| (u32::from(n) * u32::from(self.max_percent) / 100) as u16;
        let w = screen.width.saturating_sub(4).min(pct(screen.width));
        let h = screen.height.saturating_sub(3).min(pct(screen.height));
        let (area, inner) = Frame::new(w, h)
            .title(&self.title)
            .meta(Some(&self.scope))
            .placement(Placement::Center)
            .dismiss(self.dismiss)
            .render(screen, buf, ctx);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        // columns: as many as fit at 36 cells each
        let col_w = 36u16;
        let cols = (inner.width / col_w).clamp(1, 3) as usize;
        // flatten sections into column blocks
        let mut blocks: Vec<Vec<(String, String, bool)>> = vec![];
        for (title, rows) in &self.sections {
            let mut b = vec![(title.clone(), String::new(), true)];
            for (k, a) in rows {
                b.push((k.clone(), a.clone(), false));
            }
            b.push((String::new(), String::new(), false));
            blocks.push(b);
        }
        // distribute blocks round-robin into columns
        let mut columns: Vec<Vec<(String, String, bool)>> = vec![vec![]; cols];
        for (i, b) in blocks.into_iter().enumerate() {
            columns[i % cols].extend(b);
        }
        let total = columns.iter().map(Vec::len).max().unwrap_or(0);
        let body_h = inner.height.saturating_sub(1);
        self.scroll.set_content(total);
        self.scroll.set_viewport(body_h as usize);
        ctx.scrollable(self.id, inner);
        for (ci, col) in columns.iter().enumerate() {
            let x = inner.x + ci as u16 * col_w;
            for (k, i) in self.scroll.visible_range().enumerate() {
                let Some((key, action, heading)) = col.get(i) else {
                    continue;
                };
                let y = inner.y + k as u16;
                if *heading {
                    buf.set_string(x, y, key, t.secondary().bg(bg).add_modifier(Modifier::BOLD));
                } else {
                    buf.set_string(
                        x,
                        y,
                        truncate(&t.glyphs.key_label(key), 12),
                        t.key_hint_key().bg(bg),
                    );
                    buf.set_string(
                        x + 13,
                        y,
                        truncate(action, (col_w - 15) as usize),
                        t.key_hint_action().bg(bg),
                    );
                }
            }
        }
        if self.scroll.overflows() {
            crate::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    inner.y,
                    (area.right() - 2).saturating_sub(area.x),
                    body_h,
                ),
                &self.scroll,
            );
            scrollbar::render_vertical(
                Rect::new(area.right() - 2, inner.y, 1, body_h),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
            let pos = scrollbar::position_label(&self.scroll);
            let pw = width(&pos) as u16;
            // left of the close button when the frame draws one
            let reserve = if self.dismiss.shows_close() {
                crate::ui::modal::CLOSE_WIDTH + 1
            } else {
                0
            };
            buf.set_string(
                area.right().saturating_sub(pw + 4 + reserve),
                area.y,
                format!(" {pos} "),
                t.faint().bg(bg),
            );
        }
        let y = inner.bottom().saturating_sub(1);
        let bw = self.done.width();
        let room = inner.width.saturating_sub(bw + 2);
        buf.set_string(
            inner.x,
            y,
            truncate(&self.hint, room as usize),
            t.faint().bg(bg),
        );
        self.done.render(
            Rect::new(inner.right().saturating_sub(bw), y, bw, 1),
            buf,
            ctx,
            bg,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use crate::ui::popup::Dismiss;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn renders_sections_and_closes_per_policy() {
        let mut h = HelpOverlay::new(
            WidgetId::of("help"),
            "Manager",
            vec![("Move", vec![("↑↓", "Select"), ("Enter", "Open")])],
        );
        let screen = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(screen);
        let theme = Theme::junie();
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        h.render(screen, &mut buf, &mut ctx);
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("Keyboard shortcuts") && text.contains("Select"));
        let esc = Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        };
        let mut kept = HelpOverlay::new(WidgetId::of("h2"), "", vec![]).dismiss(Dismiss::NEVER);
        assert_eq!(kept.on_key(&esc), Outcome::Consumed);
        assert!(!kept.closed);
        h.on_key(&esc);
        assert!(h.closed);
    }
}
