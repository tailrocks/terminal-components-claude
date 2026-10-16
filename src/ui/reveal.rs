//! Reveal the full text of a truncated cell.
//!
//! Text cut to fit its space keeps the `…` ellipsis as the truncation
//! signal, and stays cut: rows never re-flow on their own. The full text is
//! one gesture away, following desktop conventions:
//!
//! - **Pointer.** Resting the pointer on a truncated cell shows an
//!   *expansion tooltip* (the macOS behaviour for clipped table and list
//!   cells): the full text drawn over the cell, starting at the same
//!   column, wrapped to the screen width when it is longer than a line and
//!   clamped on screen. It goes through the overlay layer, so it is never
//!   clipped by its container, never shifts layout and never claims a hit
//!   region: clicks land on the cell beneath exactly as before.
//! - **Keyboard.** The focused widget's cursor row expands with the reveal
//!   key (default `→`, the Finder list "disclose" key) and collapses with
//!   `←` or Esc. While open the reveal follows the cursor, like Quick Look
//!   follows the selection, so a list of cut values can be read with the
//!   arrow keys. Keyboard use hides pointer hover until the pointer moves
//!   ([`Interaction::live_hover`](crate::ui::ctx::Interaction::live_hover)).
//!
//! Only one reveal is visible per frame: the live pointer's beats the
//! keyboard's, otherwise the first registered wins. Copying a value is the
//! owner's business and is unaffected (it always copies the full value).
//!
//! A widget that truncates text calls [`truncated`] (pointer only) or
//! [`RevealState::cell`] (pointer and keyboard) with the cell range the text
//! was cut to and the full text. [`Reveal`] is the policy: a global default
//! on [`RenderCtx::reveal`] and an optional per-widget override.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::ColorLevel;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{self, Placement};
use crate::ui::text::{truncate, width, wrap};

/// How the full text of a truncated cell can be revealed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reveal {
    /// Resting the pointer on a truncated cell shows its expansion tooltip.
    pub hover: bool,
    /// The key that expands the focused cursor row (`←` and Esc collapse
    /// it). `None` turns the keyboard reveal off.
    pub keyboard: Option<KeyCode>,
}

impl Reveal {
    /// No reveal at all: truncated text stays cut.
    pub const OFF: Self = Self {
        hover: false,
        keyboard: None,
    };

    pub const fn hover(mut self, on: bool) -> Self {
        self.hover = on;
        self
    }

    pub const fn keyboard(mut self, key: Option<KeyCode>) -> Self {
        self.keyboard = key;
        self
    }

    /// The reveal key as a key-hint label (`→`, `Space`, `i`).
    pub fn key_label(&self) -> Option<&'static str> {
        const PRINTABLE: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        Some(match self.keyboard? {
            KeyCode::Right => "→",
            KeyCode::Left => "←",
            KeyCode::Up => "↑",
            KeyCode::Down => "↓",
            KeyCode::Enter => "Enter",
            KeyCode::Tab => "Tab",
            KeyCode::Char(' ') => "Space",
            KeyCode::Char(c) => match PRINTABLE.find(c) {
                Some(i) if c.is_ascii() => &PRINTABLE[i..i + 1],
                _ => "key",
            },
            _ => "key",
        })
    }
}

impl Default for Reveal {
    /// Pointer hover on, keyboard on `→`.
    fn default() -> Self {
        Self {
            hover: true,
            keyboard: Some(KeyCode::Right),
        }
    }
}

/// A widget's keyboard reveal: its policy override and whether the cursor
/// row is expanded. Owners call [`RevealState::begin`] at the start of
/// render, [`RevealState::cell`] per truncated cell and
/// [`RevealState::on_key`] before their own key handling.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RevealState {
    /// Per-widget policy; `None` follows [`RenderCtx::reveal`].
    pub config: Option<Reveal>,
    /// The cursor row is expanded (stays open while the cursor moves).
    pub open: bool,
    /// The policy in effect at the last render.
    effective: Reveal,
    /// The focused cursor row was truncated at the last render: the reveal
    /// key has something to show.
    available: bool,
}

impl RevealState {
    pub fn new(config: Option<Reveal>) -> Self {
        Self {
            config,
            effective: config.unwrap_or_default(),
            ..Self::default()
        }
    }

    /// The policy in effect (as of the last render).
    pub fn effective(&self) -> Reveal {
        self.effective
    }

    /// The focused cursor row was cut at the last render, so the reveal key
    /// does something: owners show its key hint only then.
    pub fn available(&self) -> bool {
        self.available && self.effective.keyboard.is_some()
    }

    /// The key hint for the reveal key, when it applies: `(key, label)`.
    pub fn hint(&self) -> Option<(&'static str, &'static str)> {
        if !self.available() {
            return None;
        }
        let key = if self.open {
            "←"
        } else {
            self.effective.key_label()?
        };
        Some((key, if self.open { "Collapse" } else { "Full text" }))
    }

    /// Start a frame: resolve the policy against the global default.
    pub fn begin(&mut self, ctx: &RenderCtx) {
        self.effective = self.config.unwrap_or(ctx.reveal);
        self.available = false;
        if self.effective.keyboard.is_none() {
            self.open = false;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// The reveal key opens, `←` and Esc close. Keys are only taken when
    /// they do something visible: the reveal key on a cut cursor row, a
    /// close key while the reveal shows. Everything else is the owner's.
    pub fn on_key(&mut self, key: &Key) -> Option<Outcome> {
        let open_key = self.effective.keyboard?;
        if !key.plain() {
            return None;
        }
        if key.code == open_key && self.available && !self.open {
            self.open = true;
            return Some(Outcome::Changed);
        }
        if self.open && self.available && matches!(key.code, KeyCode::Left | KeyCode::Esc) {
            self.open = false;
            return Some(Outcome::Changed);
        }
        None
    }

    /// Register a cell showing `full` cut to `area`. `cursor` is true for
    /// the focused widget's cursor row: it is the one the keyboard reveal
    /// expands. Returns whether the text was cut.
    pub fn cell(
        &mut self,
        ctx: &mut RenderCtx,
        id: WidgetId,
        area: Rect,
        full: &str,
        cursor: bool,
    ) -> bool {
        let cut = is_cut(area, full);
        if cut && cursor && !ctx.inert {
            self.available = true;
            if self.open && self.effective.keyboard.is_some() {
                show(ctx, area, full, Source::Keyboard);
            }
        }
        if self.effective.hover {
            hover(ctx, id, area, full);
        }
        cut
    }
}

/// Register a pointer-only reveal for a cell showing `full` cut to `area`,
/// under `config` (or the global default). Returns whether the text was cut.
pub fn truncated(
    ctx: &mut RenderCtx,
    config: Option<Reveal>,
    id: WidgetId,
    area: Rect,
    full: &str,
) -> bool {
    let cfg = config.unwrap_or(ctx.reveal);
    if cfg.hover {
        hover(ctx, id, area, full)
    } else {
        is_cut(area, full)
    }
}

fn is_cut(area: Rect, full: &str) -> bool {
    !area.is_empty() && width(full) > area.width as usize
}

fn hover(ctx: &mut RenderCtx, id: WidgetId, area: Rect, full: &str) -> bool {
    let cut = is_cut(area, full);
    if cut && !ctx.inert && ctx.interaction.live_hover() == Some(id) {
        show(ctx, area, full, Source::Pointer);
    }
    cut
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    Keyboard,
    Pointer,
}

#[derive(Debug, Clone)]
struct Tip {
    anchor: Rect,
    text: String,
    source: Source,
}

/// The frame's single reveal, drawn by one queued overlay.
#[derive(Debug, Default)]
pub(crate) struct Slot {
    tip: Option<Tip>,
    queued: bool,
}

fn show(ctx: &mut RenderCtx, anchor: Rect, full: &str, source: Source) {
    let slot = &mut ctx.reveal_slot;
    match &slot.tip {
        Some(tip) if !(tip.source == Source::Keyboard && source == Source::Pointer) => return,
        _ => {}
    }
    slot.tip = Some(Tip {
        anchor,
        text: full.to_owned(),
        source,
    });
    if !slot.queued {
        slot.queued = true;
        ctx.overlay(|buf, ctx| {
            ctx.reveal_slot.queued = false;
            if let Some(tip) = ctx.reveal_slot.tip.take() {
                draw(buf, ctx, &tip);
            }
        });
    }
}

/// Where a reveal of `text` for `anchor` lands on `screen`, and its lines.
/// The first line starts at the anchor's column (one cell of padding to
/// its left), over the anchor itself; a long text wraps to the screen and
/// the box moves left, then up, to stay on screen.
fn layout(screen: Rect, anchor: Rect, text: &str) -> (Rect, Vec<String>) {
    let room = screen.width.saturating_sub(2).max(1) as usize;
    let lines = wrap(text, room);
    let w = lines.iter().map(|l| width(l)).max().unwrap_or(0) as u16 + 2;
    let h = lines.len() as u16;
    let at = Rect::new(anchor.x.saturating_sub(1), anchor.y, 0, 0);
    (popup::place(screen, at, w, h, Placement::Below), lines)
}

fn draw(buf: &mut Buffer, ctx: &mut RenderCtx, tip: &Tip) {
    let screen = *buf.area();
    let t = ctx.theme;
    let (area, lines) = layout(screen, tip.anchor, &tip.text);
    let area = area.intersection(screen);
    if area.is_empty() {
        return;
    }
    let mut st = Style::new().fg(t.text_primary).bg(t.popover);
    if t.level == ColorLevel::Mono {
        st = st.add_modifier(Modifier::REVERSED);
    }
    fill(buf, area, st);
    let rows = area.height as usize;
    let inner_w = area.width.saturating_sub(2) as usize;
    for (k, line) in lines.iter().take(rows).enumerate() {
        let mut line = truncate(line, inner_w);
        // more text than the screen holds: the last row says so
        if k + 1 == rows && lines.len() > rows {
            line = truncate(&format!("{line}…"), inner_w);
            if !line.ends_with('…') {
                line.push('…');
            }
        }
        buf.set_string(area.x + 1, area.y + k as u16, &line, st);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn row_text(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    fn frame(
        w: u16,
        h: u16,
        interaction: Interaction,
        draw: impl FnOnce(&mut Buffer, &mut RenderCtx),
    ) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
        let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
        draw(&mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        buf
    }

    const FULL: &str = "jackin-preview · deterministic redesign preview";

    fn cut_cell(buf: &mut Buffer, area: Rect) {
        buf.set_string(
            area.x,
            area.y,
            truncate(FULL, area.width as usize),
            Style::new(),
        );
    }

    #[test]
    fn hovering_a_cut_cell_shows_its_full_text_over_it() {
        let id = WidgetId::of("cell");
        let area = Rect::new(4, 2, 20, 1);
        let hover = Interaction {
            hover: Some(id),
            ..Default::default()
        };
        let buf = frame(80, 6, hover, |buf, ctx| {
            cut_cell(buf, area);
            assert!(truncated(ctx, None, id, area, FULL));
        });
        assert_eq!(row_text(&buf, 2).trim(), FULL);
        // starts at the cell's own column: the cut text is expanded in place
        assert_eq!(buf[(4, 2)].symbol(), "j");
        assert_eq!(buf[(4, 2)].bg, Theme::junie().popover);
    }

    #[test]
    fn nothing_shows_without_hover_or_when_the_text_fits() {
        let id = WidgetId::of("cell");
        let area = Rect::new(4, 2, 20, 1);
        let buf = frame(80, 6, Interaction::default(), |buf, ctx| {
            cut_cell(buf, area);
            assert!(truncated(ctx, None, id, area, FULL));
        });
        assert!(!row_text(&buf, 2).contains("redesign preview"));
        let wide = Rect::new(4, 2, 60, 1);
        let hover = Interaction {
            hover: Some(id),
            ..Default::default()
        };
        let buf = frame(80, 6, hover, |buf, ctx| {
            buf.set_string(4, 2, FULL, Style::new());
            assert!(!truncated(ctx, None, id, wide, FULL));
        });
        assert_ne!(
            buf[(3, 2)].bg,
            Theme::junie().popover,
            "no tooltip for text that fits"
        );
    }

    #[test]
    fn suppressed_hover_and_the_off_policy_show_nothing() {
        let id = WidgetId::of("cell");
        let area = Rect::new(4, 2, 20, 1);
        let suppressed = Interaction {
            hover: Some(id),
            hover_suppressed: true,
            ..Default::default()
        };
        let buf = frame(80, 6, suppressed, |_, ctx| {
            truncated(ctx, None, id, area, FULL);
        });
        assert!(!row_text(&buf, 2).contains("redesign preview"));
        let hover = Interaction {
            hover: Some(id),
            ..Default::default()
        };
        let buf = frame(80, 6, hover, |_, ctx| {
            truncated(ctx, Some(Reveal::OFF), id, area, FULL);
        });
        assert!(!row_text(&buf, 2).contains("redesign preview"));
        let buf = frame(80, 6, hover, |_, ctx| {
            ctx.reveal = Reveal::OFF;
            truncated(ctx, None, id, area, FULL);
        });
        assert!(
            !row_text(&buf, 2).contains("redesign preview"),
            "global default off"
        );
    }

    #[test]
    fn a_long_text_wraps_and_stays_on_screen() {
        let id = WidgetId::of("cell");
        let long = "word ".repeat(30);
        let area = Rect::new(30, 9, 8, 1);
        let hover = Interaction {
            hover: Some(id),
            ..Default::default()
        };
        let buf = frame(40, 10, hover, |_, ctx| {
            truncated(ctx, None, id, area, long.trim());
        });
        let tooltip: Vec<u16> = (0..10)
            .filter(|&y| (0..40).any(|x| buf[(x, y)].bg == Theme::junie().popover))
            .collect();
        assert!(tooltip.len() > 1, "wrapped over several rows");
        let text: String = tooltip
            .iter()
            .map(|&y| row_text(&buf, y).trim().to_owned())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(text, long.trim());
        // the bottom row anchor pushed the box up, inside the screen
        let (rect, _) = layout(Rect::new(0, 0, 40, 10), area, long.trim());
        assert!(rect.right() <= 40 && rect.bottom() <= 10);
    }

    #[test]
    fn a_text_taller_than_the_screen_ends_with_an_ellipsis() {
        let id = WidgetId::of("cell");
        let long = "word ".repeat(200);
        let hover = Interaction {
            hover: Some(id),
            ..Default::default()
        };
        let buf = frame(20, 4, hover, |_, ctx| {
            truncated(ctx, None, id, Rect::new(2, 1, 6, 1), long.trim());
        });
        assert!(row_text(&buf, 3).trim_end().ends_with('…'));
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    #[test]
    fn the_reveal_key_expands_the_cut_cursor_row_and_left_collapses_it() {
        let id = WidgetId::of("row");
        let area = Rect::new(4, 2, 20, 1);
        let mut state = RevealState::default();
        let render = |state: &mut RevealState| {
            frame(80, 6, Interaction::default(), |buf, ctx| {
                state.begin(ctx);
                cut_cell(buf, area);
                state.cell(ctx, id, area, FULL, true);
            })
        };
        let buf = render(&mut state);
        assert!(!row_text(&buf, 2).contains("redesign preview"));
        assert!(state.available());
        assert_eq!(state.hint(), Some(("→", "Full text")));
        assert_eq!(state.on_key(&key(KeyCode::Right)), Some(Outcome::Changed));
        let buf = render(&mut state);
        assert_eq!(row_text(&buf, 2).trim(), FULL);
        assert_eq!(state.on_key(&key(KeyCode::Left)), Some(Outcome::Changed));
        let buf = render(&mut state);
        assert!(!row_text(&buf, 2).contains("redesign preview"));
    }

    #[test]
    fn the_reveal_key_passes_through_when_nothing_is_cut() {
        let mut state = RevealState::default();
        frame(80, 6, Interaction::default(), |_, ctx| {
            state.begin(ctx);
            state.cell(ctx, WidgetId::of("row"), Rect::new(0, 0, 70, 1), FULL, true);
        });
        assert!(!state.available());
        assert_eq!(state.on_key(&key(KeyCode::Right)), None);
        assert_eq!(state.on_key(&key(KeyCode::Esc)), None);
    }

    #[test]
    fn the_live_pointer_wins_over_the_keyboard_reveal() {
        let row = WidgetId::of("row");
        let other = WidgetId::of("other");
        let mut state = RevealState {
            open: true,
            ..RevealState::default()
        };
        let hover = Interaction {
            hover: Some(other),
            ..Default::default()
        };
        let buf = frame(80, 6, hover, |_, ctx| {
            state.begin(ctx);
            state.cell(
                ctx,
                row,
                Rect::new(4, 1, 10, 1),
                "keyboard row text long",
                true,
            );
            state.cell(
                ctx,
                other,
                Rect::new(4, 3, 10, 1),
                "pointer row text long",
                false,
            );
        });
        assert!(row_text(&buf, 3).contains("pointer row text long"));
        assert!(!row_text(&buf, 1).contains("keyboard row text long"));
    }
}
