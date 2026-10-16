//! Chip bar: a row of removable, toggleable chips (filters, tags). One
//! focus stop; ← → move the cursor, Enter activates, Delete removes.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::theme::{ButtonKind, Kind};
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::{truncate, width};

/// How many items of a horizontal run fit in `avail` cells (`widths` each,
/// `gap` cells between them), and the `+N` marker naming the ones that do
/// not. When the whole run fits the marker is `None`; otherwise as many
/// items as leave room for the marker are kept, so a cut run always says
/// how much it hides instead of ending silently.
pub fn overflow_run(widths: &[u16], avail: u16, gap: u16) -> (usize, Option<String>) {
    let total: u32 = widths.iter().map(|&w| w as u32).sum::<u32>()
        + gap as u32 * widths.len().saturating_sub(1) as u32;
    if total <= avail as u32 {
        return (widths.len(), None);
    }
    let n = widths.len();
    for k in (0..n).rev() {
        let marker = format!("+{}", n - k);
        let used: u32 = widths[..k]
            .iter()
            .map(|&w| w as u32 + gap as u32)
            .sum::<u32>()
            + width(&marker) as u32;
        if used <= avail as u32 {
            return (k, Some(marker));
        }
    }
    (0, None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    pub label: String,
    pub enabled: bool,
    pub removable: bool,
    pub error: bool,
}

impl Chip {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            enabled: true,
            removable: true,
            error: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChipBar {
    pub id: WidgetId,
    pub chips: Vec<Chip>,
    pub cursor: usize,
    pub add_label: Option<String>,
    /// Leading label such as `match all ▾` (clickable, emits `Toggle`).
    pub lead: Option<String>,
    pub area: Rect,
    /// Chips that do not fit are counted in a muted `+N` marker after the
    /// last one shown. Default true; off, the run just ends.
    pub overflow_marker: bool,
    /// Which keys move between chips (default [`NavKeys::DEFAULT`]: ←/→,
    /// `h`/`l`, Home/End, `g`/`G`).
    pub nav: NavKeys,
    /// The bar's action keys (default [`ChipAction::default_keymap`]).
    pub keymap: KeyMap<ChipAction>,
}

/// A [`ChipBar`] action a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipAction {
    /// Remove the cursor chip (when removable). Default Delete, Backspace
    /// and `x`.
    Remove,
    /// Ask to add a chip. Default `+`.
    Add,
    /// Ask to clear every chip. Default `X`.
    ClearAll,
}

impl Actions for ChipAction {
    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::key(KeyCode::Delete), ChipAction::Remove),
            (KeyBinding::key(KeyCode::Backspace), ChipAction::Remove),
            (KeyBinding::char('x'), ChipAction::Remove),
            (KeyBinding::char('+'), ChipAction::Add),
            (KeyBinding::char('X'), ChipAction::ClearAll),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipEvent {
    Activate(usize),
    Toggle(usize),
    Remove(usize),
    Add,
    Lead,
    ClearAll,
}

impl ChipBar {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            chips: vec![],
            cursor: 0,
            add_label: Some("+ Add filter".into()),
            lead: None,
            area: Rect::ZERO,
            overflow_marker: true,
            nav: NavKeys::DEFAULT,
            keymap: ChipAction::default_keymap(),
        }
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<ChipAction>) -> Self {
        self.keymap = keymap;
        self
    }

    /// Show (default) or hide the `+N` count of chips that do not fit.
    pub fn overflow_marker(mut self, on: bool) -> Self {
        self.overflow_marker = on;
        self
    }

    pub fn chip_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn close_id(&self, i: usize) -> WidgetId {
        self.id.child(i).sub("x")
    }
    pub fn add_id(&self) -> WidgetId {
        self.id.sub("add")
    }
    pub fn lead_id(&self) -> WidgetId {
        self.id.sub("lead")
    }

    /// Cursor positions: 0..n chips, n = add button.
    fn stops(&self) -> usize {
        self.chips.len() + usize::from(self.add_label.is_some())
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ChipEvent>) {
        let n = self.stops();
        if n == 0 {
            return (Outcome::Ignored, None);
        }
        self.cursor = self.cursor.min(n - 1);
        let action = lookup(&self.keymap, key);
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) && action.is_none() {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        match action {
            Some(ChipAction::Remove)
                if self.cursor < self.chips.len() && self.chips[self.cursor].removable =>
            {
                return (Outcome::Changed, Some(ChipEvent::Remove(self.cursor)));
            }
            Some(ChipAction::Add) => return (Outcome::Changed, Some(ChipEvent::Add)),
            Some(ChipAction::ClearAll) => return (Outcome::Changed, Some(ChipEvent::ClearAll)),
            _ => {}
        }
        let to = match self.nav.action(key).filter(|_| key.plain()) {
            Some(Nav::Left) => Some(self.cursor.saturating_sub(1)),
            Some(Nav::Right) => Some((self.cursor + 1).min(n - 1)),
            Some(Nav::Home) => Some(0),
            Some(Nav::End) => Some(n - 1),
            _ => None,
        };
        if let Some(to) = to {
            self.cursor = to;
            return (Outcome::Changed, None);
        }
        match key.code {
            KeyCode::Enter => {
                if self.cursor < self.chips.len() {
                    (Outcome::Changed, Some(ChipEvent::Activate(self.cursor)))
                } else {
                    (Outcome::Changed, Some(ChipEvent::Add))
                }
            }
            KeyCode::Char(' ') if self.cursor < self.chips.len() => {
                (Outcome::Changed, Some(ChipEvent::Toggle(self.cursor)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<ChipEvent>) {
        if id == self.add_id() {
            self.cursor = self.chips.len();
            return (Outcome::Changed, Some(ChipEvent::Add));
        }
        if id == self.lead_id() {
            return (Outcome::Changed, Some(ChipEvent::Lead));
        }
        for i in 0..self.chips.len() {
            if self.close_id(i) == id {
                return (Outcome::Changed, Some(ChipEvent::Remove(i)));
            }
            if self.chip_id(i) == id {
                self.cursor = i;
                return (Outcome::Changed, Some(ChipEvent::Activate(i)));
            }
        }
        (Outcome::Ignored, None)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == self.add_id()
            || id == self.lead_id()
            || (0..self.chips.len()).any(|i| self.chip_id(i) == id || self.close_id(i) == id)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let plane = t.plane_of(bg);
        let mut x = area.x;
        let y = area.y;
        if let Some(lead) = &self.lead {
            let lid = self.lead_id();
            let mut s = ctx.state(lid);
            s.focused = false;
            let st = t.interactive(Kind::Title, s, plane).style;
            let text = format!(" {lead} ");
            buf.set_string(x, y, &text, st);
            ctx.clickable(lid, Rect::new(x, y, width(&text) as u16, 1));
            x += width(&text) as u16 + 1;
        }
        let chip_w =
            |c: &Chip| 1 + width(&c.label) as u16 + 1 + if c.removable { 2 } else { 0 } + 1;
        let widths: Vec<u16> = self.chips.iter().map(chip_w).collect();
        let (shown, marker) = overflow_run(&widths, area.right().saturating_sub(x), 1);
        for (i, chip) in self.chips.iter().enumerate() {
            if i >= shown {
                if let Some(m) = marker.as_ref().filter(|_| self.overflow_marker) {
                    buf.set_string(x, y, m, t.muted().bg(bg));
                    x += width(m) as u16 + 1;
                }
                break;
            }
            let cid = self.chip_id(i);
            // the × is its own control: hovering it does not hover the chip
            let mut s = ctx.child_state(self.id, cid, i == self.cursor);
            s.focused = focused && i == self.cursor;
            s.selected = chip.enabled;
            let it = t.interactive(Kind::Chip, s, plane);
            let mut st = it.style;
            if chip.error {
                st = st.fg(t.legible(t.error, st.bg.unwrap_or(bg)));
            }
            let label_w = width(&chip.label) as u16;
            let w = widths[i];
            let r = Rect::new(x, y, w, 1);
            fill(buf, r, st);
            buf.set_string(x, y, it.gutter, it.gutter_style);
            buf.set_string(x + 1, y, truncate(&chip.label, label_w as usize), st);
            if chip.removable {
                let xid = self.close_id(i);
                let xs = t.close_glyph(ctx.state(xid), st);
                buf.set_string(x + 2 + label_w, y, t.glyphs.close, xs);
                ctx.clickable(cid, r);
                ctx.clickable(xid, Rect::new(x + 2 + label_w, y, 1, 1));
            } else {
                ctx.clickable(cid, r);
            }
            x += w + 1;
        }
        if let Some(add) = &self.add_label {
            let aid = self.add_id();
            let mut s = ctx.child_state(self.id, aid, self.cursor == self.chips.len());
            s.focused = focused && self.cursor == self.chips.len();
            let it = t.interactive(Kind::Button(ButtonKind::Subtle), s, plane);
            let st = it.style;
            let w = width(add) as u16 + 2;
            if x + w <= area.right() {
                let r = Rect::new(x, y, w, 1);
                fill(buf, r, st);
                buf.set_string(x, y, it.gutter, it.gutter_style);
                buf.set_string(x + 1, y, add, st);
                ctx.clickable(aid, r);
            }
        }
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Plane, Theme};
    use crate::ui::ctx::Interaction;

    fn draw(bar: &mut ChipBar, t: &Theme, interaction: Interaction) -> Buffer {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(t, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        bar.render(Rect::new(0, 0, 40, 1), &mut buf, &mut ctx, t.canvas);
        buf
    }

    fn bar() -> ChipBar {
        let mut bar = ChipBar::new(WidgetId::of("chips"));
        let mut off = Chip::new("beta");
        off.enabled = false;
        bar.chips = vec![Chip::new("alpha"), off];
        bar
    }

    #[test]
    fn hovering_the_close_glyph_does_not_hover_the_chip_and_it_presses() {
        let t = Theme::junie();
        let mut b = bar();
        let x_id = b.close_id(0);
        // " alpha × ": the × sits after the label and a space
        let rest = draw(&mut b, &t, Interaction::default());
        let x = (0..40).find(|&x| rest[(x, 0)].symbol() == "×").unwrap();
        let hover = draw(
            &mut b,
            &t,
            Interaction {
                hover: Some(x_id),
                ..Default::default()
            },
        );
        assert_eq!(hover[(2, 0)].bg, rest[(2, 0)].bg, "chip body stays at rest");
        assert_ne!(hover[(x, 0)].bg, rest[(x, 0)].bg, "× shows its own hover");
        let pressed = draw(
            &mut b,
            &t,
            Interaction {
                hover: Some(x_id),
                pressed: Some(x_id),
                ..Default::default()
            },
        );
        assert_ne!(pressed[(x, 0)].bg, hover[(x, 0)].bg, "× shows a press");
        assert_eq!(
            pressed[(x, 0)].bg,
            t.ladder(Plane::Raised).pressed,
            "× presses on its chip's plane"
        );
    }

    #[test]
    fn an_off_chip_label_is_readable_and_distinct_from_an_on_chip_at_every_level() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let mut b = bar();
            let buf = draw(&mut b, &t, Interaction::default());
            let on = &buf[(1, 0)];
            let x = (0..40).find(|&x| buf[(x, 0)].symbol() == "b").unwrap();
            let off = &buf[(x, 0)];
            assert_ne!(off.fg, off.bg, "{level:?}: off label on its plane");
            assert_ne!(
                (on.fg, on.modifier),
                (off.fg, off.modifier),
                "{level:?}: on and off chips look alike"
            );
            if level == ColorLevel::Mono {
                assert_ne!(on.modifier, off.modifier, "mono: on and off differ");
            }
        }
    }
}

#[cfg(test)]
mod overflow_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn a_run_that_does_not_fit_keeps_room_for_its_count() {
        assert_eq!(overflow_run(&[5, 5, 5], 17, 1), (3, None));
        // 5+1+5+1 = 12, "+1" needs 2 more: 14 <= 16
        assert_eq!(overflow_run(&[5, 5, 5], 16, 1), (2, Some("+1".into())));
        // only the marker fits
        assert_eq!(overflow_run(&[9, 9], 4, 1), (0, Some("+2".into())));
        assert_eq!(overflow_run(&[9, 9], 1, 1), (0, None));
    }

    fn render(bar: &mut ChipBar, w: u16) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, w, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf, &mut ctx, theme.canvas);
        (0..w).map(|x| buf[(x, 0)].symbol().to_owned()).collect()
    }

    #[test]
    fn chips_that_do_not_fit_are_counted_with_a_marker() {
        let mut bar = ChipBar::new(WidgetId::of("c"));
        bar.add_label = None;
        bar.chips = ["alpha", "beta", "gamma", "delta"]
            .iter()
            .map(|l| Chip::new(l))
            .collect();
        let line = render(&mut bar, 24);
        assert!(line.contains("alpha"), "{line:?}");
        assert!(line.contains("+3") || line.contains("+2"), "{line:?}");
        assert!(!line.contains('…'), "{line:?}");
        let mut off = bar.clone().overflow_marker(false);
        let line = render(&mut off, 24);
        assert!(!line.contains('+'), "{line:?}");
    }
}
