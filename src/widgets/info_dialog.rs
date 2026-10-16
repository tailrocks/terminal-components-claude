//! A read-only information dialog: an intro, a property list whose rows
//! can be copied, an optional detail block and action buttons. Moved from
//! jackin-preview so every app can use it.

use crate::core::event::{Key, Outcome};
use crate::core::focus::{Focus, FocusRing};
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::scroll::ScrollState;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::truncate;
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::props::{Prop, PropsAction, PropsList};
use crate::widgets::scrollbar;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoResult {
    Closed,
    Copy(String),
    /// Extra action button index fired.
    Action(usize),
}

/// Read-only facts with copyable rows and explicit actions.
pub struct InfoDialog {
    /// The `×` its frame draws when its [`Dismiss`] policy lets Esc close
    /// it ([`Dismiss::shows_close`]); a click on it means Esc. Default
    /// [`IconButton::close`] (three-cell plate, `Close · Esc` tooltip);
    /// every modal takes a custom one the same way
    /// ([`crate::ui::layer::Overlay::close_button`]).
    ///
    /// [`Dismiss`]: crate::ui::popup::Dismiss
    /// [`Dismiss::shows_close`]: crate::ui::popup::Dismiss::shows_close
    /// [`IconButton::close`]: crate::widgets::icon_button::IconButton::close
    pub close_button: crate::widgets::icon_button::IconButton,
    /// Which dismissals close it. Default `Dismiss::LIGHT`.
    pub dismiss: crate::ui::popup::Dismiss,
    pub id: WidgetId,
    pub title: String,
    pub meta: Option<String>,
    pub intro: Vec<(String, Tone)>,
    pub props: PropsList,
    pub detail: Vec<String>,
    pub detail_scroll: ScrollState,
    pub actions: Vec<Button>,
    pub close: Button,
    pub width: u16,
    pub result: Option<InfoResult>,
    pub area: Rect,
    /// Copies of copyable values by row index.
    pub copy_values: Vec<Option<String>>,
    pub error_title: bool,
    /// Which keys scroll the detail text (default [`NavKeys::DEFAULT`]).
    pub nav: NavKeys,
    /// The dialog's action keys (default [`InfoCmd::default_keymap`]).
    pub keymap: KeyMap<InfoCmd>,
    /// The theme's strings as of the last render; a Close label still
    /// reading them follows a change ([`crate::theme::Strings::follow`]).
    strings: crate::theme::Strings,
}

/// An [`InfoDialog`] action a key can be bound to. Esc and Enter always
/// close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoCmd {
    /// Close the dialog. Default `q`.
    Close,
    /// Copy the first copyable value. Default `y`.
    Copy,
}

impl Actions for InfoCmd {
    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::char('q'), InfoCmd::Close),
            (KeyBinding::char('y'), InfoCmd::Copy),
        ]
    }
}

impl InfoDialog {
    /// See [`Self::close_button`].
    pub fn close_button(mut self, button: crate::widgets::icon_button::IconButton) -> Self {
        self.close_button = button;
        self
    }

    pub fn new(id: WidgetId, title: &str, props: Vec<Prop>) -> Self {
        let copy_values = props
            .iter()
            .map(|p| p.copyable.then(|| p.value.clone()))
            .collect();
        Self {
            dismiss: crate::ui::popup::Dismiss::LIGHT,
            close_button: crate::widgets::icon_button::IconButton::close(crate::ui::modal::CLOSE),
            id,
            title: title.to_owned(),
            meta: None,
            intro: vec![],
            props: PropsList::new(id.sub("props"), props),
            detail: vec![],
            detail_scroll: ScrollState::default(),
            actions: vec![],
            close: Button::secondary(id.sub("close"), crate::theme::Strings::DEFAULT.close),
            strings: crate::theme::Strings::DEFAULT,
            width: 66,
            result: None,
            area: Rect::ZERO,
            copy_values,
            error_title: false,
            nav: NavKeys::DEFAULT,
            keymap: InfoCmd::default_keymap(),
        }
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<InfoCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn intro(mut self, lines: Vec<(String, Tone)>) -> Self {
        self.intro = lines;
        self
    }

    pub fn detail(mut self, lines: Vec<String>) -> Self {
        self.detail = lines;
        self
    }

    pub fn action(mut self, b: Button) -> Self {
        self.actions.push(b);
        self
    }

    pub fn width(mut self, w: u16) -> Self {
        self.width = w;
        self
    }

    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.to_owned());
        self
    }

    pub fn error(mut self) -> Self {
        self.error_title = true;
        self
    }

    pub fn initial_focus(&self) -> WidgetId {
        if self.props.props.is_empty() {
            self.close.id
        } else {
            self.props.id
        }
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing) -> Outcome {
        let cur = focus.current();
        if cur == Some(self.props.id) {
            let (o, ev) = self.props.on_key(key);
            match ev {
                Some(PropsAction::Copy(i)) => {
                    if let Some(Some(v)) = self.copy_values.get(i) {
                        self.result = Some(InfoResult::Copy(v.clone()));
                    }
                    return Outcome::Changed;
                }
                Some(PropsAction::Activate(i)) => {
                    if let Some(Some(v)) = self.copy_values.get(i) {
                        self.result = Some(InfoResult::Copy(v.clone()));
                        return Outcome::Changed;
                    }
                }
                None => {}
            }
            if o.consumed() {
                return o;
            }
            // detail scrolling from the list
            match key.code {
                KeyCode::PageDown => {
                    self.detail_scroll.page_down();
                    return Outcome::Changed;
                }
                KeyCode::PageUp => {
                    self.detail_scroll.page_up();
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        for i in 0..self.actions.len() {
            if cur == Some(self.actions[i].id) {
                let (o, fired) = self.actions[i].on_key(key);
                if fired {
                    self.result = Some(InfoResult::Action(i));
                    return Outcome::Changed;
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        if cur == Some(self.close.id) {
            let (o, fired) = self.close.on_key(key);
            if fired {
                self.result = Some(InfoResult::Closed);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        match lookup(&self.keymap, key) {
            Some(InfoCmd::Close) => {
                self.result = Some(InfoResult::Closed);
                return Outcome::Changed;
            }
            Some(InfoCmd::Copy) => {
                // copy the first copyable value
                if let Some(Some(v)) = self.copy_values.iter().find(|v| v.is_some()) {
                    self.result = Some(InfoResult::Copy(v.clone()));
                }
                return Outcome::Changed;
            }
            None => {}
        }
        match self.nav.action(key) {
            Some(Nav::Down) => {
                self.detail_scroll.scroll_by(1);
                return Outcome::Changed;
            }
            Some(Nav::Up) => {
                self.detail_scroll.scroll_by(-1);
                return Outcome::Changed;
            }
            _ => {}
        }
        match key.code {
            KeyCode::Esc => {
                self.result = Some(InfoResult::Closed);
                Outcome::Changed
            }
            KeyCode::Enter => {
                self.result = Some(InfoResult::Closed);
                Outcome::Changed
            }
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_click(&mut self, id: WidgetId, pos: Position, focus: &mut Focus) -> Outcome {
        if let Some(i) = self.props.locate(id) {
            focus.focus(self.props.id);
            let (o, _) = self.props.on_click(i);
            if let Some(Some(v)) = self.copy_values.get(i) {
                self.result = Some(InfoResult::Copy(v.clone()));
            }
            return o;
        }
        if id == scrollbar::id_for(self.props.id) {
            return self.props.on_scrollbar(pos);
        }
        if id == scrollbar::id_for(self.id) {
            let track = Rect::new(
                self.area.right() - 2,
                self.area.y + 1,
                1,
                self.area.height.saturating_sub(2),
            );
            self.detail_scroll.scroll_to(scrollbar::offset_for_click(
                track,
                pos,
                &self.detail_scroll,
            ));
            return Outcome::Changed;
        }
        for i in 0..self.actions.len() {
            if self.actions[i].id == id {
                focus.focus(id);
                if self.actions[i].on_click() {
                    self.result = Some(InfoResult::Action(i));
                }
                return Outcome::Changed;
            }
        }
        if id == self.close.id {
            focus.focus(id);
            if self.close.on_click() {
                self.result = Some(InfoResult::Closed);
            }
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.detail.is_empty() {
            self.props.on_wheel(delta)
        } else {
            self.detail_scroll.scroll_by(delta as isize);
            Outcome::Changed
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let s = ctx.theme.strings;
        let old = std::mem::replace(&mut self.strings, s);
        crate::theme::Strings::follow(&mut self.close.label, old.close, s.close);
        let intro_h = self.intro.len() as u16 + u16::from(!self.intro.is_empty());
        let props_h = self.props.props.len() as u16;
        let detail_h = if self.detail.is_empty() {
            0
        } else {
            (self.detail.len() as u16).min(8) + 1
        };
        let h = 2 + 2 + intro_h + props_h + detail_h + 2;
        let title = if self.error_title {
            format!("! {}", self.title)
        } else {
            self.title.clone()
        };
        let (area, inner) = crate::ui::modal::Frame::new(self.width, h)
            .title(&title)
            .meta(self.meta.as_deref())
            .overlay(&*self)
            .render(screen, buf, ctx);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        if self.error_title {
            buf.set_string(
                area.x + 3,
                area.y,
                "!",
                t.error_fg().bg(bg).add_modifier(Modifier::BOLD),
            );
        }
        let mut y = inner.y;
        for (l, tone) in &self.intro {
            buf.set_string(
                inner.x,
                y,
                truncate(l, inner.width as usize),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
            y += 1;
        }
        if !self.intro.is_empty() {
            y += 1;
        }
        let avail = inner.bottom().saturating_sub(y + 2);
        let ph = props_h.min(avail);
        if ph > 0 {
            self.props
                .render(Rect::new(inner.x - 1, y, inner.width + 1, ph), buf, ctx, bg);
            y += ph;
        }
        if !self.detail.is_empty() {
            y += 1;
            let dh = inner.bottom().saturating_sub(y + 2);
            let wrapped: Vec<String> = self
                .detail
                .iter()
                .flat_map(|l| crate::ui::text::wrap(l, inner.width.saturating_sub(2) as usize))
                .collect();
            self.detail_scroll.set_content(wrapped.len());
            self.detail_scroll.set_viewport(dh as usize);
            for (k, i) in self.detail_scroll.visible_range().enumerate() {
                buf.set_string(inner.x, y + k as u16, &wrapped[i], t.secondary().bg(bg));
            }
            if self.detail_scroll.overflows() {
                crate::ui::fade::scroll_edges(
                    buf,
                    ctx,
                    Rect::new(area.x, y, (area.right() - 2).saturating_sub(area.x), dh),
                    &self.detail_scroll,
                );
                scrollbar::render_vertical(
                    Rect::new(area.right() - 2, y, 1, dh),
                    buf,
                    ctx,
                    self.id,
                    &self.detail_scroll,
                    true,
                );
            }
        }
        let ay = inner.bottom().saturating_sub(1);
        let mut widths: Vec<u16> = self.actions.iter().map(|b| b.width()).collect();
        widths.push(self.close.width());
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        for (i, b) in self.actions.iter_mut().enumerate() {
            b.render(rects[i], buf, ctx, bg);
        }
        self.close.render(rects[self.actions.len()], buf, ctx, bg);
        for b in &self.actions {
            ctx.hits.register(b.id, b.area);
        }
        ctx.hits.register(self.close.id, self.close.area);
        for i in self.props.scroll.visible_range() {
            ctx.hits.raise(self.props.row_id(i));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn screen_text(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn a_library_info_dialog_shows_its_facts_and_closes_on_escape() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut d = InfoDialog::new(
            WidgetId::of("info"),
            "Instance",
            vec![Prop::new("Image", "ubuntu:24.04").copyable()],
        );
        {
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 90, 30));
            d.render(Rect::new(0, 0, 90, 30), &mut buf, &mut ctx);
            assert!(screen_text(&buf).contains("ubuntu:24.04"));
        }
        let mut focus = Focus::default();
        focus.focus(d.initial_focus());
        d.on_key(&key(KeyCode::Esc), &mut focus, &ring);
        assert_eq!(d.result, Some(InfoResult::Closed));
    }
}
