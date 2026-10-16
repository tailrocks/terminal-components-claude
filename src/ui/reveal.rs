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
//! - **In place.** With [`RevealMode::InlineWrap`] the keyboard reveal
//!   wraps the cursor row in place instead, accordion style: the row grows
//!   to every line of its text and the rows below move down. Owners that
//!   can re-flow their rows ([`ListBox`](crate::widgets::list::ListBox),
//!   [`PropsList`](crate::widgets::props::PropsList))
//!   draw it through [`RevealState::inline`]; the others keep the tooltip.
//! - **Long text.** A reveal longer than its rows (the screen, or
//!   [`Reveal::max_rows`]) shows `↑ N more` / `↓ N more` markers and
//!   scrolls with PgUp/PgDn and the wheel ([`Reveal::scroll`]), instead of
//!   ending in `…`.
//!
//! Only one reveal is visible per frame: the live pointer's beats the
//! keyboard's, otherwise the first registered wins. Copying a value is the
//! owner's business and is unaffected (it always copies the full value).
//!
//! A widget that truncates text calls [`truncated`] (pointer only) or
//! [`RevealState::cell`] (pointer and keyboard) with the cell range the text
//! was cut to and the full text. [`Reveal`] is the policy: a global default
//! on [`RenderCtx::reveal`] and an optional per-widget override.

use std::rc::Rc;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::ColorLevel;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{self, Placement};
use crate::ui::text::{truncate, wider_than, width, wrap};

/// How the keyboard reveal shows the full text of the cursor row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RevealMode {
    /// An expansion tooltip over the row, through the overlay layer: the
    /// rows around it never move (the default).
    #[default]
    Tooltip,
    /// The cursor row wraps in place, accordion style: it grows to show
    /// every line and the rows below it move down. Owners that cannot
    /// re-flow their rows (tables, grids) fall back to the tooltip.
    /// Pointer hover always uses the tooltip: rows must not move under the
    /// pointer.
    InlineWrap,
}

/// How the full text of a truncated cell can be revealed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reveal {
    /// Resting the pointer on a truncated cell shows its expansion tooltip.
    pub hover: bool,
    /// The key that expands the focused cursor row (`←` and Esc collapse
    /// it). `None` turns the keyboard reveal off.
    pub keyboard: Option<KeyCode>,
    /// How the keyboard reveal shows (default [`RevealMode::Tooltip`]).
    pub mode: RevealMode,
    /// Most rows a reveal takes before its text scrolls. `0` (the default)
    /// takes as many as fit: the screen for a tooltip, the owner's viewport
    /// for an inline reveal.
    pub max_rows: u16,
    /// A reveal longer than its rows scrolls with PgUp/PgDn and the wheel
    /// (default on). Off, it stays at its first lines and ends with
    /// `↓ N more`.
    pub scroll: bool,
    /// Lines one wheel notch scrolls a pointer-only tooltip (one shown by
    /// [`truncated`] or [`queue_cut`], which no widget state owns; see
    /// [`on_pointer_wheel`]). Default 3, the pointer's default. A
    /// [`RevealState`] owner passes its own wheel delta instead.
    pub wheel_lines: u16,
    /// How long the pointer rests on a cut cell before its expansion
    /// tooltip shows (ms). Default 300: sweeping across a list does not
    /// flash a tooltip per row. The keyboard reveal is immediate.
    pub delay_ms: u64,
    /// At either end of a scrolling tooltip the wheel goes on to the
    /// content beneath (default on), instead of stalling there.
    pub wheel_passes_at_ends: bool,
}

impl Reveal {
    /// No reveal at all: truncated text stays cut.
    pub const OFF: Self = Self {
        hover: false,
        keyboard: None,
        mode: RevealMode::Tooltip,
        max_rows: 0,
        scroll: false,
        wheel_lines: 3,
        delay_ms: 300,
        wheel_passes_at_ends: true,
    };

    pub const fn hover(mut self, on: bool) -> Self {
        self.hover = on;
        self
    }

    pub const fn keyboard(mut self, key: Option<KeyCode>) -> Self {
        self.keyboard = key;
        self
    }

    pub const fn mode(mut self, mode: RevealMode) -> Self {
        self.mode = mode;
        self
    }

    pub const fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = rows;
        self
    }

    pub const fn scroll(mut self, on: bool) -> Self {
        self.scroll = on;
        self
    }

    pub const fn wheel_lines(mut self, lines: u16) -> Self {
        self.wheel_lines = lines;
        self
    }

    pub const fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    pub const fn wheel_passes_at_ends(mut self, on: bool) -> Self {
        self.wheel_passes_at_ends = on;
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
    /// Pointer hover on, keyboard on `→` as a tooltip, long text scrolls.
    fn default() -> Self {
        Self {
            hover: true,
            keyboard: Some(KeyCode::Right),
            mode: RevealMode::Tooltip,
            max_rows: 0,
            scroll: true,
            wheel_lines: 3,
            delay_ms: 300,
            wheel_passes_at_ends: true,
        }
    }
}

/// What the last drawn reveal of a [`RevealState`] could scroll: written
/// when the reveal is laid out (in the overlay, for a tooltip), read by the
/// next key or wheel event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Extent {
    /// Furthest scroll offset; `0`: the text fits, nothing scrolls.
    max: u16,
    /// Lines one page moves.
    page: u16,
}

type SharedExtent = std::rc::Rc<std::cell::Cell<Extent>>;

/// A widget's keyboard reveal: its policy override and whether the cursor
/// row is expanded. Owners call [`RevealState::begin`] at the start of
/// render, [`RevealState::cell`] per truncated cell (and
/// [`RevealState::inline`] for the cursor row first, when they can re-flow
/// it), and [`RevealState::on_key`] / [`RevealState::on_wheel`] before
/// their own handling.
#[derive(Debug, Clone, Default)]
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
    /// The owner drew the cursor row's reveal inline this frame.
    inlined: bool,
    /// First line shown of a reveal longer than its rows.
    scroll: u16,
    /// The cell the scroll offset belongs to: another cell starts at its
    /// first line.
    target: Option<WidgetId>,
    /// Shared with the queued tooltip, which lays itself out at frame end.
    extent: SharedExtent,
    /// The theme's hint labels (`reveal_open`, `reveal_close`) as of the
    /// last frame; `None` before the first is the default copy.
    labels: Option<(&'static str, &'static str)>,
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
        let d = crate::theme::Strings::DEFAULT;
        let (open, close) = self.labels.unwrap_or((d.reveal_open, d.reveal_close));
        Some((key, if self.open { close } else { open }))
    }

    /// The reveal drawn last is longer than its rows and scrolls.
    pub fn scrolls(&self) -> bool {
        self.effective.scroll && self.extent.get().max > 0
    }

    /// Start a frame: resolve the policy against the global default.
    pub fn begin(&mut self, ctx: &RenderCtx) {
        self.effective = self.config.unwrap_or(ctx.reveal);
        let s = &ctx.theme.strings;
        self.labels = Some((s.reveal_open, s.reveal_close));
        self.available = false;
        self.inlined = false;
        self.extent.set(Extent::default());
        if self.effective.keyboard.is_none() {
            self.open = false;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// The reveal key opens, `←` and Esc close, PgUp/PgDn scroll a reveal
    /// longer than its rows. Keys are only taken when they do something
    /// visible: the reveal key on a cut cursor row, a close or scroll key
    /// while the reveal shows. Everything else is the owner's.
    pub fn on_key(&mut self, key: &Key) -> Option<Outcome> {
        let open_key = self.effective.keyboard?;
        if !key.plain() {
            return None;
        }
        if key.code == open_key && self.available && !self.open {
            self.open = true;
            self.scroll = 0;
            return Some(Outcome::Changed);
        }
        if self.open && self.available {
            match key.code {
                KeyCode::Left | KeyCode::Esc => {
                    self.open = false;
                    return Some(Outcome::Changed);
                }
                KeyCode::PageDown | KeyCode::PageUp if self.scrolls() => {
                    let page = self.extent.get().page.max(1) as i32;
                    let delta = if key.code == KeyCode::PageDown {
                        page
                    } else {
                        -page
                    };
                    return Some(self.scroll_by(delta));
                }
                _ => {}
            }
        }
        None
    }

    /// The wheel scrolls this widget's reveal when it is longer than its
    /// rows (the open keyboard reveal, or the pointer's tooltip). `None`
    /// when nothing scrolls: the owner scrolls its own content.
    pub fn on_wheel(&mut self, delta: i32) -> Option<Outcome> {
        if !self.scrolls() {
            return None;
        }
        let out = self.scroll_by(delta);
        // at an end the wheel goes on to the content, per policy
        if out == Outcome::Consumed && self.effective.wheel_passes_at_ends {
            return None;
        }
        Some(out)
    }

    fn scroll_by(&mut self, delta: i32) -> Outcome {
        let max = self.extent.get().max as i32;
        let next = (self.scroll as i32 + delta).clamp(0, max) as u16;
        if next == self.scroll {
            // at an end the reveal still holds the event, so the content
            // beneath does not jump while the text is being read
            Outcome::Consumed
        } else {
            self.scroll = next;
            Outcome::Changed
        }
    }

    /// A reveal of `id` is about to show: the scroll offset belongs to one
    /// cell, so another starts at its first line.
    fn track(&mut self, id: WidgetId) {
        if self.target != Some(id) {
            self.target = Some(id);
            self.scroll = 0;
        }
    }

    /// The rows an inline reveal ([`RevealMode::InlineWrap`]) of the cursor
    /// row takes, when it applies: the keyboard reveal is open and `full`
    /// does not fit `width`. Call it before drawing the cursor row; the
    /// owner then draws the returned rows in place of the cut label, one per
    /// screen row, and skips [`RevealState::cell`] for that row. `room` is
    /// the most rows the owner can give the row; a longer text scrolls.
    /// `None`: draw the row as usual (a cut cursor row still counts as
    /// available, so the reveal key works).
    pub fn inline(
        &mut self,
        ctx: &mut RenderCtx,
        id: WidgetId,
        width: u16,
        full: &str,
        cursor: bool,
        room: u16,
    ) -> Option<Vec<RevealLine>> {
        if !cursor || ctx.inert || !is_cut(Rect::new(0, 0, width, 1), full) {
            return None;
        }
        self.available = true;
        self.advertise(ctx);
        if !self.open
            || self.effective.keyboard.is_none()
            || self.effective.mode != RevealMode::InlineWrap
        {
            return None;
        }
        self.inlined = true;
        self.track(id);
        let lines = wrapped(id, full, width.max(1) as usize);
        let rows = cap(room, self.effective.max_rows) as usize;
        let scroll = if self.effective.scroll {
            self.scroll
        } else {
            0
        };
        let (shown, extent) = window(&lines, rows, scroll as usize, ctx.theme);
        self.extent.set(extent);
        self.scroll = self.scroll.min(extent.max);
        Some(shown)
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
            self.advertise(ctx);
            if self.open && self.effective.keyboard.is_some() && !self.inlined {
                self.track(id);
                let feed = self.feed();
                show(ctx, id, area, full, Source::Keyboard, feed);
            }
        }
        if self.effective.hover
            && cut
            && !ctx.inert
            && ctx.interaction.live_hover() == Some(id)
            && hover_ready(ctx, self.effective.delay_ms)
        {
            self.track(id);
            let feed = self.feed();
            show(ctx, id, area, full, Source::Pointer, feed);
        }
        cut
    }

    /// The focused cursor row is cut: publish the reveal key hint for the
    /// app's footer ([`RenderCtx::reveal_hint`]).
    fn advertise(&self, ctx: &mut RenderCtx) {
        if let Some(h) = self.hint() {
            ctx.reveal_hint = Some(h);
        }
    }

    fn feed(&self) -> Feed {
        Feed {
            scroll: if self.effective.scroll {
                self.scroll
            } else {
                0
            },
            max_rows: self.effective.max_rows,
            extent: Some(self.extent.clone()),
            pointer_only: false,
        }
    }
}

/// One row of a reveal: a line of the text, or a `↑ N more` / `↓ N more`
/// marker for lines scrolled out of view (`more`; drawn muted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealLine {
    pub text: String,
    pub more: bool,
}

/// The rows a reveal may take: `room`, capped by `max_rows` when set.
fn cap(room: u16, max_rows: u16) -> u16 {
    if max_rows > 0 {
        room.min(max_rows)
    } else {
        room
    }
}

/// The `rows` of `lines` shown at `scroll`, with markers for what is
/// scrolled out above and below, and how far it scrolls.
fn window(
    lines: &[String],
    rows: usize,
    scroll: usize,
    t: &crate::theme::Theme,
) -> (Vec<RevealLine>, Extent) {
    let line = |s: &String| RevealLine {
        text: s.clone(),
        more: false,
    };
    let more = |n: usize, up: bool| RevealLine {
        text: format!(
            "{} {}",
            if up {
                t.glyphs.arrows[0]
            } else {
                t.glyphs.arrows[1]
            },
            crate::theme::Strings::fill(t.strings.more_count, &[&n])
        ),
        more: true,
    };
    let n = lines.len();
    if rows == 0 {
        return (Vec::new(), Extent::default());
    }
    if n <= rows {
        return (lines.iter().map(line).collect(), Extent::default());
    }
    if rows == 1 {
        let max = n - 1;
        let extent = Extent {
            max: max as u16,
            page: 1,
        };
        return (vec![line(&lines[scroll.min(max)])], extent);
    }
    // at the top the last row is the ↓ marker, at the end the first row is
    // the ↑ marker, in between both (when a line fits between them)
    let max = n - (rows - 1);
    let o = scroll.min(max);
    let extent = Extent {
        max: max as u16,
        page: rows.saturating_sub(2).max(1) as u16,
    };
    let mut out = Vec::with_capacity(rows);
    if o == 0 {
        out.extend(lines[..rows - 1].iter().map(line));
        out.push(more(n - (rows - 1), false));
    } else if o == max {
        out.push(more(o, true));
        out.extend(lines[o..].iter().map(line));
    } else if rows >= 3 {
        let end = o + rows - 2;
        out.push(more(o, true));
        out.extend(lines[o..end].iter().map(line));
        out.push(more(n - end, false));
    } else {
        out.push(line(&lines[o]));
        out.push(more(n - o - 1, false));
    }
    (out, extent)
}

/// Register a pointer-only reveal for a cell showing `full` cut to `area`,
/// under `config` (or the global default). Returns whether the text was
/// cut. A tooltip longer than the screen shows `↓ N more` and scrolls with
/// the wheel over it ([`on_pointer_wheel`], which the runtime calls before
/// the app sees the wheel); off with [`Reveal::scroll`].
pub fn truncated(
    ctx: &mut RenderCtx,
    config: Option<Reveal>,
    id: WidgetId,
    area: Rect,
    full: &str,
) -> bool {
    let cfg = config.unwrap_or(ctx.reveal);
    let cut = is_cut(area, full);
    if cfg.hover
        && cut
        && !ctx.inert
        && ctx.interaction.live_hover() == Some(id)
        && hover_ready(ctx, cfg.delay_ms)
    {
        let feed = POINTER.with(|p| {
            let mut p = p.borrow_mut();
            if p.target != Some(id) {
                p.target = Some(id);
                p.scroll = 0;
            }
            p.wheel_lines = cfg.wheel_lines;
            p.scrolls = cfg.scroll;
            p.passes = cfg.wheel_passes_at_ends;
            Feed {
                scroll: if cfg.scroll { p.scroll } else { 0 },
                max_rows: cfg.max_rows,
                extent: Some(p.extent.clone()),
                pointer_only: true,
            }
        });
        show(ctx, id, area, full, Source::Pointer, feed);
    }
    cut
}

/// The scroll state of the pointer-only tooltip ([`truncated`]): no widget
/// owns it, and only one reveal shows per frame.
#[derive(Debug, Default)]
struct PointerTip {
    /// The cell the scroll offset belongs to.
    target: Option<WidgetId>,
    scroll: u16,
    extent: SharedExtent,
    /// Where the tooltip was drawn in the last frame, if it was.
    drawn: Option<Rect>,
    wheel_lines: u16,
    scrolls: bool,
    /// [`Reveal::wheel_passes_at_ends`] of the drawn tooltip.
    passes: bool,
}

thread_local! {
    static POINTER: std::cell::RefCell<PointerTip> =
        std::cell::RefCell::new(PointerTip::default());
}

/// A frame starts: a pointer-only tooltip is under the pointer only if
/// this frame draws it again.
pub(crate) fn begin_frame() {
    POINTER.with(|p| p.borrow_mut().drawn = None);
    HOVER_WAIT.with(|w| w.set(None));
    FRAME.with(|f| f.set(f.get().wrapping_add(1)));
}

thread_local! {
    /// The shortest wait (ms) before hover content held back this frame
    /// may show ([`hover_ready`]); `None` when nothing waits.
    static HOVER_WAIT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    /// Frames started, so a reader tells one frame's wait from the next.
    static FRAME: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Whether hover content that waits `delay_ms` for the pointer to rest may
/// show this frame, per [`crate::ui::ctx::Interaction::hover_ms`]. When it
/// must still wait, the time left is recorded for [`hover_wait`], so
/// the runtime draws again once it has passed (no input arrives to cause
/// a frame while the pointer rests).
pub fn hover_ready(ctx: &RenderCtx, delay_ms: u64) -> bool {
    let Some(rested) = ctx.interaction.hover_ms else {
        return true;
    };
    if rested >= delay_ms {
        return true;
    }
    let left = delay_ms - rested;
    HOVER_WAIT.with(|w| w.set(Some(w.get().map_or(left, |v| v.min(left)))));
    false
}

/// How long (ms) after the last frame hover content it held back may show
/// ([`hover_ready`]), with that frame's number: the runtime draws again
/// then.
pub fn hover_wait() -> Option<(u64, u64)> {
    let frame = FRAME.with(|f| f.get());
    HOVER_WAIT.with(|w| w.get()).map(|ms| (frame, ms))
}

/// The wheel over a pointer-only tooltip ([`truncated`], [`queue_cut`])
/// longer than its rows scrolls the tooltip, the way a [`RevealState`]
/// owner scrolls its own. `None` when the event is not a vertical wheel
/// over such a tooltip, as last drawn: the app handles it as usual. At
/// either end the wheel goes on to the content beneath
/// ([`Reveal::wheel_passes_at_ends`], default on); off, the tooltip holds
/// it (`Consumed`).
/// [`crate::runtime`] calls this before dispatching each input.
pub fn on_pointer_wheel(input: &crate::core::event::Input) -> Option<Outcome> {
    let crate::core::event::Input::Mouse(m) = input else {
        return None;
    };
    let (0, dy) = m.wheel()? else {
        return None;
    };
    if !m.mods.is_empty() {
        return None;
    }
    POINTER.with(|p| {
        let mut p = p.borrow_mut();
        let extent = p.extent.get();
        if !p.scrolls || extent.max == 0 || !p.drawn?.contains(m.pos) {
            return None;
        }
        let next =
            (p.scroll as i32 + dy * p.wheel_lines.max(1) as i32).clamp(0, extent.max as i32) as u16;
        if next == p.scroll && p.passes {
            // at an end the wheel goes on to the content beneath
            return None;
        }
        Some(if next == p.scroll {
            Outcome::Consumed
        } else {
            p.scroll = next;
            Outcome::Changed
        })
    })
}

/// Put the frame's reveal key hint ([`RenderCtx::reveal_hint`]: `→ Full
/// text` while the focused cursor row is cut, `← Collapse` while it shows)
/// into a footer's hints. It goes after the first hint (usually the move
/// keys), so a narrow footer that drops hints from the right keeps it, and
/// is skipped when its key is already advertised. `key_of` reads a hint's
/// key, `make` builds one, so any hint type works.
pub fn advertise<T>(
    hints: &mut Vec<T>,
    reveal: Option<(&'static str, &'static str)>,
    key_of: impl Fn(&T) -> &str,
    make: impl FnOnce(&'static str, &'static str) -> T,
) {
    let Some((key, label)) = reveal else {
        return;
    };
    if hints.iter().any(|h| key_of(h) == key) {
        return;
    }
    hints.insert(hints.len().min(1), make(key, label));
}

thread_local! {
    /// Cut text drawn this frame by code that renders without a
    /// [`RenderCtx`] (panel title rows, static property sheets),
    /// registered at the next overlay flush.
    static CUTS: std::cell::RefCell<Vec<(Rect, String)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Most cut texts queued per frame: chrome rendered over and over without
/// a frame to flush it never grows the queue.
const CUTS_MAX: usize = 64;

/// Queue text shown cut to `area` whose full text is `full`, for code that
/// draws without a [`RenderCtx`] (panel titles, [`props::render`] sheets).
/// At the next overlay flush its cells become a hover-only region (clicks
/// still reach what is beneath) that shows the expansion tooltip under the
/// global [`RenderCtx::reveal`] policy. Text that fits is ignored. There
/// is no keyboard reveal: such text has no focus or cursor row.
///
/// [`props::render`]: crate::widgets::props::render
pub fn queue_cut(area: Rect, full: &str) {
    if !is_cut(area, full) {
        return;
    }
    CUTS.with(|q| {
        let mut q = q.borrow_mut();
        q.retain(|(a, _)| *a != area);
        if q.len() < CUTS_MAX {
            q.push((area, full.to_owned()));
        }
    });
}

/// Drop text queued outside a frame (a new frame starts clean).
pub(crate) fn clear_cuts() {
    CUTS.with(|q| q.borrow_mut().clear());
}

/// The hover id of queued cut text at `area`: by position, so two panels
/// with the same title reveal each their own.
pub fn cut_id(area: Rect) -> WidgetId {
    WidgetId::of("junie.reveal.cut").child(((area.x as usize) << 16) | area.y as usize)
}

/// Register the queued cut text as hover regions with their reveal.
/// Behind a modal (`inert`) it is dropped: nothing beneath it reveals.
pub(crate) fn flush_cuts(ctx: &mut RenderCtx) {
    let cuts = CUTS.with(|q| std::mem::take(&mut *q.borrow_mut()));
    if ctx.inert {
        return;
    }
    for (area, full) in cuts {
        let id = cut_id(area);
        if ctx.reveal.hover {
            ctx.hits.register_hover(id, area);
        }
        truncated(ctx, None, id, area, &full);
    }
}

fn is_cut(area: Rect, full: &str) -> bool {
    !area.is_empty() && wider_than(full, area.width as usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    Keyboard,
    Pointer,
}

/// Where a tooltip starts reading, its row cap, and where it reports how
/// far it could scroll.
#[derive(Debug, Clone, Default)]
struct Feed {
    scroll: u16,
    max_rows: u16,
    extent: Option<SharedExtent>,
    /// A pointer-only tooltip: its drawn box is where the wheel scrolls it.
    pointer_only: bool,
}

#[derive(Debug, Clone)]
struct Tip {
    anchor: Rect,
    id: WidgetId,
    /// Shared with the wrap cache: a reveal resting on the same value
    /// frame after frame neither copies nor re-wraps it.
    text: Rc<str>,
    source: Source,
    feed: Feed,
}

/// The frame's single reveal, drawn by one queued overlay.
#[derive(Debug, Default)]
pub(crate) struct Slot {
    tip: Option<Tip>,
    queued: bool,
}

/// The wrapped lines of the last reveal drawn: a reveal stays on screen
/// for many frames (a resting pointer, an open keyboard reveal), and
/// wrapping a long value costs time linear in its length.
#[derive(Debug)]
struct Wrapped {
    id: WidgetId,
    text: Rc<str>,
    width: usize,
    lines: Rc<Vec<String>>,
}

thread_local! {
    /// One entry: only one reveal shows per frame.
    static WRAPPED: std::cell::RefCell<Option<Wrapped>> =
        const { std::cell::RefCell::new(None) };
    /// Texts wrapped for a reveal so far (a complexity guard).
    static WRAPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// `full` as the shared text of the cached reveal when it is the same
/// value (a length check, then one comparison: no copy), else a new copy.
fn shared(full: &str) -> Rc<str> {
    WRAPPED.with(|w| match &*w.borrow() {
        Some(c) if c.text.len() == full.len() && *c.text == *full => c.text.clone(),
        _ => Rc::from(full),
    })
}

/// The lines of `full` wrapped to `width` for the reveal of `id`, from the
/// cache when the cell, its text and the width are unchanged. Any change
/// (another cell, an edited value, a resize) wraps again.
fn wrapped(id: WidgetId, full: &str, width: usize) -> Rc<Vec<String>> {
    WRAPPED.with(|w| {
        let mut w = w.borrow_mut();
        if let Some(c) = &*w
            && c.id == id
            && c.width == width
            && c.text.len() == full.len()
            && (std::ptr::eq(c.text.as_ptr(), full.as_ptr()) || *c.text == *full)
        {
            return c.lines.clone();
        }
        WRAPS.with(|n| n.set(n.get() + 1));
        let text = match &*w {
            Some(c) if *c.text == *full => c.text.clone(),
            _ => Rc::from(full),
        };
        let lines = Rc::new(wrap(full, width));
        *w = Some(Wrapped {
            id,
            text,
            width,
            lines: lines.clone(),
        });
        lines
    })
}

fn show(ctx: &mut RenderCtx, id: WidgetId, anchor: Rect, full: &str, source: Source, feed: Feed) {
    let slot = &mut ctx.reveal_slot;
    match &slot.tip {
        Some(tip) if !(tip.source == Source::Keyboard && source == Source::Pointer) => return,
        _ => {}
    }
    slot.tip = Some(Tip {
        anchor,
        id,
        text: shared(full),
        source,
        feed,
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

/// Where a reveal of `text` for `anchor` lands on `screen`, and its rows.
/// The first line starts at the anchor's column (one cell of padding to
/// its left), over the anchor itself; a long text wraps to the screen and
/// the box moves left, then up, to stay on screen. Text taller than the
/// box shows the window at `feed.scroll`, with `↑`/`↓ N more` markers.
fn layout(
    screen: Rect,
    anchor: Rect,
    id: WidgetId,
    text: &str,
    feed: &Feed,
    t: &crate::theme::Theme,
) -> (Rect, Vec<RevealLine>) {
    let room = screen.width.saturating_sub(2).max(1) as usize;
    let lines = wrapped(id, text, room);
    let rows = cap(screen.height, feed.max_rows) as usize;
    let (shown, extent) = window(&lines, rows, feed.scroll as usize, t);
    if let Some(e) = &feed.extent {
        e.set(extent);
    }
    let w = shown.iter().map(|l| width(&l.text)).max().unwrap_or(0) as u16 + 2;
    let h = shown.len() as u16;
    let at = Rect::new(anchor.x.saturating_sub(1), anchor.y, 0, 0);
    // a tooltip covers the very cell it explains: no edge margin
    let mut area = popup::place_with(screen, at, w, h, Placement::Below, popup::Place::FLUSH);
    // the window is laid out for `h` rows: when the room below the anchor
    // is shorter the box keeps its height and moves up, so a marker row is
    // never cut off
    if area.height < h {
        area.height = h.min(screen.height);
        area.y = screen.bottom().saturating_sub(area.height);
    }
    (area, shown)
}

fn draw(buf: &mut Buffer, ctx: &mut RenderCtx, tip: &Tip) {
    let screen = *buf.area();
    let t = ctx.theme;
    let (area, lines) = layout(screen, tip.anchor, tip.id, &tip.text, &tip.feed, t);
    let area = area.intersection(screen);
    if area.is_empty() {
        return;
    }
    if tip.feed.pointer_only {
        POINTER.with(|p| p.borrow_mut().drawn = Some(area));
    }
    // the popover plane, the strongest neutral; over content already on
    // that plane (a menu) it steps down a plane, so the tooltip always
    // reads as its own layer
    let on_popover = area
        .positions()
        .any(|p| buf.cell(p).is_some_and(|c| c.bg == t.popover));
    let plane = if on_popover {
        t.surface_overlay
    } else {
        t.popover
    };
    let mut st = Style::new().fg(t.text_primary).bg(plane);
    if t.level == ColorLevel::Mono {
        st = st.add_modifier(Modifier::REVERSED);
    }
    fill(buf, area, st);
    let inner_w = area.width.saturating_sub(2) as usize;
    for (k, line) in lines.iter().take(area.height as usize).enumerate() {
        let style = if line.more { st.fg(t.text_muted) } else { st };
        buf.set_string(
            area.x + 1,
            area.y + k as u16,
            truncate(&line.text, inner_w),
            style,
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

    /// Guard (performance): a reveal resting on a long value wraps it
    /// once, not every frame; a change of cell, text or width wraps again.
    #[test]
    fn a_resting_reveal_wraps_its_text_once() {
        let wraps = || WRAPS.with(std::cell::Cell::get);
        let id = WidgetId::of("cell");
        let area = Rect::new(4, 2, 20, 1);
        let long = "lorem ipsum ".repeat(64 * 1024 / 12);
        let show = |w: u16, text: &str, id: WidgetId| {
            let hover = Interaction {
                hover: Some(id),
                ..Default::default()
            };
            frame(w, 20, hover, |_, ctx| {
                truncated(ctx, None, id, area, text);
            })
        };
        let start = wraps();
        let first = show(80, &long, id);
        for _ in 0..10 {
            assert_eq!(show(80, &long, id), first, "the same tooltip");
        }
        assert_eq!(wraps() - start, 1, "eleven frames, one wrap");
        let edited = format!("{long}!");
        show(80, &edited, id);
        assert_eq!(wraps() - start, 2, "an edited value wraps again");
        show(60, &edited, id);
        assert_eq!(wraps() - start, 3, "a resize wraps again");
        show(60, &edited, WidgetId::of("other"));
        assert_eq!(wraps() - start, 4, "another cell wraps its own text");
        show(60, &edited, WidgetId::of("other"));
        assert_eq!(wraps() - start, 4);
        // the open keyboard reveal, inline or as a tooltip, uses the cache
        let mut state = RevealState::new(Some(Reveal::default().mode(RevealMode::InlineWrap)));
        state.open = true;
        let focus = Interaction {
            focus: Some(id),
            ..Default::default()
        };
        for _ in 0..5 {
            frame(80, 20, focus, |_, ctx| {
                state.begin(ctx);
                assert!(state.inline(ctx, id, 20, &long, true, 10).is_some());
            });
        }
        assert_eq!(wraps() - start, 5, "five inline frames, one wrap");
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

    /// The expansion tooltip waits for the pointer to rest
    /// ([`Reveal::delay_ms`], 300) and says when to draw again (it showed
    /// at once, flashing one tooltip per row while sweeping a list).
    #[test]
    fn the_hover_reveal_waits_for_the_pointer_to_rest() {
        let id = WidgetId::of("cell");
        let area = Rect::new(4, 2, 20, 1);
        let at = |ms: Option<u64>, cfg: Option<Reveal>| {
            let hover = Interaction {
                hover: Some(id),
                hover_ms: ms,
                ..Default::default()
            };
            let buf = frame(80, 6, hover, |buf, ctx| {
                cut_cell(buf, area);
                truncated(ctx, cfg, id, area, FULL);
            });
            (row_text(&buf, 2).trim() == FULL, hover_wait().map(|w| w.1))
        };
        assert_eq!(at(Some(0), None), (false, Some(300)));
        assert_eq!(at(Some(120), None), (false, Some(180)));
        assert_eq!(at(Some(300), None), (true, None));
        assert_eq!(at(None, None), (true, None), "no clock: at once");
        assert_eq!(
            at(Some(0), Some(Reveal::default().delay_ms(0))),
            (true, None)
        );
        // the keyboard reveal does not wait
        let mut state = RevealState::default();
        let focus = Interaction {
            focus: Some(id),
            hover_ms: Some(0),
            ..Default::default()
        };
        frame(80, 6, focus, |buf, ctx| {
            state.begin(ctx);
            cut_cell(buf, area);
            state.cell(ctx, id, area, FULL, true);
        });
        state.open = true;
        let buf = frame(80, 6, focus, |buf, ctx| {
            state.begin(ctx);
            cut_cell(buf, area);
            state.cell(ctx, id, area, FULL, true);
        });
        assert_eq!(row_text(&buf, 2).trim(), FULL);
    }

    /// At the ends of a scrolling pointer tooltip the wheel goes on to the
    /// content beneath (it stalled there).
    #[test]
    fn the_wheel_passes_through_at_a_pointer_tooltips_ends() {
        use crate::core::event::{Input, Mouse, MouseKind};
        use ratatui::layout::Position;
        let id = WidgetId::of("long");
        let area = Rect::new(0, 0, 10, 1);
        let text = numbered(30);
        let draw = |cfg: Reveal| {
            let hover = Interaction {
                hover: Some(id),
                ..Default::default()
            };
            frame(30, 6, hover, |_, ctx| {
                truncated(ctx, Some(cfg), id, area, &text);
            });
        };
        let wheel = |kind| Input::Mouse(Mouse::new(kind, Position::new(1, 1)));
        draw(Reveal::default());
        // at the top, up passes; down scrolls the tooltip
        assert_eq!(on_pointer_wheel(&wheel(MouseKind::WheelUp)), None);
        assert_eq!(
            on_pointer_wheel(&wheel(MouseKind::WheelDown)),
            Some(Outcome::Changed)
        );
        draw(Reveal::default());
        while on_pointer_wheel(&wheel(MouseKind::WheelDown)) == Some(Outcome::Changed) {
            draw(Reveal::default());
        }
        assert_eq!(
            on_pointer_wheel(&wheel(MouseKind::WheelDown)),
            None,
            "the end"
        );
        // the policy holds the wheel at the ends instead
        let hold = Reveal::default().wheel_passes_at_ends(false);
        draw(hold);
        assert_eq!(
            on_pointer_wheel(&wheel(MouseKind::WheelDown)),
            Some(Outcome::Consumed)
        );
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
        let (rect, _) = layout(
            Rect::new(0, 0, 40, 10),
            area,
            WidgetId::of("t"),
            long.trim(),
            &Feed::default(),
            &Theme::junie(),
        );
        assert!(rect.right() <= 40 && rect.bottom() <= 10);
    }

    #[test]
    fn a_text_taller_than_the_screen_says_how_much_more_there_is() {
        let id = WidgetId::of("cell");
        let long = "word ".repeat(200);
        let hover = Interaction {
            hover: Some(id),
            ..Default::default()
        };
        let buf = frame(20, 4, hover, |_, ctx| {
            truncated(ctx, None, id, Rect::new(2, 1, 6, 1), long.trim());
        });
        let last = row_text(&buf, 3);
        assert!(
            last.contains("↓") && last.contains("more"),
            "{:?}",
            (0..4).map(|y| row_text(&buf, y)).collect::<Vec<_>>()
        );
    }

    /// Long text on the open keyboard reveal: one frame per call.
    fn long_reveal(state: &mut RevealState, text: &str) -> Buffer {
        frame(30, 6, Interaction::default(), |_, ctx| {
            state.begin(ctx);
            state.cell(ctx, WidgetId::of("row"), Rect::new(2, 0, 8, 1), text, true);
        })
    }

    fn numbered(n: usize) -> String {
        (1..=n)
            .map(|i| format!("line{i:02}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn screen_text(buf: &Buffer) -> String {
        (0..buf.area.height)
            .map(|y| row_text(buf, y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_long_keyboard_reveal_scrolls_with_page_keys_and_the_wheel() {
        // 40 words at 28 columns wrap to many more rows than the 6 on screen
        let text = numbered(40);
        let mut state = RevealState::default();
        long_reveal(&mut state, &text);
        assert_eq!(state.on_key(&key(KeyCode::Right)), Some(Outcome::Changed));
        let buf = long_reveal(&mut state, &text);
        let first = screen_text(&buf);
        assert!(first.contains("line01"), "{first}");
        assert!(first.contains("↓") && first.contains("more"), "{first}");
        assert!(state.scrolls());
        // PgDn moves down a page: the first line scrolls out, ↑ says so
        assert_eq!(
            state.on_key(&key(KeyCode::PageDown)),
            Some(Outcome::Changed)
        );
        let buf = long_reveal(&mut state, &text);
        let paged = screen_text(&buf);
        assert!(!paged.contains("line01 "), "{paged}");
        assert!(paged.contains("↑"), "{paged}");
        // the wheel reaches the end; past it the wheel goes on to the content
        while state.on_wheel(3) == Some(Outcome::Changed) {
            long_reveal(&mut state, &text);
        }
        let buf = long_reveal(&mut state, &text);
        let end = screen_text(&buf);
        assert!(end.contains("line40"), "{end}");
        assert!(!end.contains("↓"), "{end}");
        assert_eq!(state.on_wheel(1), None, "the wheel passes at the end");
        // a policy holds it there instead
        state.config = Some(Reveal::default().wheel_passes_at_ends(false));
        long_reveal(&mut state, &text);
        assert_eq!(state.on_wheel(1), Some(Outcome::Consumed));
        // PgUp goes back
        assert_eq!(state.on_key(&key(KeyCode::PageUp)), Some(Outcome::Changed));
    }

    #[test]
    fn short_reveals_and_the_scroll_knob_leave_the_wheel_to_the_owner() {
        let mut state = RevealState::default();
        long_reveal(&mut state, FULL);
        state.on_key(&key(KeyCode::Right));
        long_reveal(&mut state, FULL);
        assert!(!state.scrolls(), "the full text fits the screen");
        assert_eq!(state.on_wheel(1), None);
        assert_eq!(state.on_key(&key(KeyCode::PageDown)), None);
        let text = numbered(40);
        let mut off = RevealState::new(Some(Reveal::default().scroll(false)));
        long_reveal(&mut off, &text);
        off.on_key(&key(KeyCode::Right));
        let buf = long_reveal(&mut off, &text);
        assert!(screen_text(&buf).contains("↓"), "still says there is more");
        assert_eq!(off.on_wheel(1), None);
    }

    #[test]
    fn max_rows_caps_the_reveal_before_it_scrolls() {
        let text = numbered(40);
        let mut state = RevealState::new(Some(Reveal::default().max_rows(3)));
        long_reveal(&mut state, &text);
        state.on_key(&key(KeyCode::Right));
        let buf = long_reveal(&mut state, &text);
        let rows = (0..6)
            .filter(|&y| (0..30).any(|x| buf[(x, y)].bg == Theme::junie().popover))
            .count();
        assert_eq!(rows, 3);
        assert!(state.scrolls());
    }

    #[test]
    fn the_window_marks_hidden_lines_above_and_below() {
        let lines: Vec<String> = (0..10).map(|i| format!("l{i}")).collect();
        let text = |v: Vec<RevealLine>| v.into_iter().map(|l| l.text).collect::<Vec<_>>();
        let (top, e) = window(&lines, 4, 0, &Theme::junie());
        assert_eq!(text(top), ["l0", "l1", "l2", "↓ 7 more"]);
        assert_eq!(e.max, 7);
        let (mid, _) = window(&lines, 4, 3, &Theme::junie());
        assert_eq!(text(mid), ["↑ 3 more", "l3", "l4", "↓ 5 more"]);
        let (end, _) = window(&lines, 4, 99, &Theme::junie());
        assert_eq!(text(end), ["↑ 7 more", "l7", "l8", "l9"]);
        let (fits, e) = window(&lines[..3], 4, 2, &Theme::junie());
        assert_eq!(text(fits), ["l0", "l1", "l2"]);
        assert_eq!(e.max, 0);
    }

    #[test]
    fn inline_mode_hands_the_cursor_row_to_its_owner_instead_of_a_tooltip() {
        let id = WidgetId::of("row");
        let mut state = RevealState::new(Some(Reveal::default().mode(RevealMode::InlineWrap)));
        let draw = |state: &mut RevealState| {
            let mut rows = None;
            let buf = frame(80, 6, Interaction::default(), |_, ctx| {
                state.begin(ctx);
                rows = state.inline(ctx, id, 20, FULL, true, 4);
                if rows.is_none() {
                    state.cell(ctx, id, Rect::new(4, 2, 20, 1), FULL, true);
                }
            });
            (buf, rows)
        };
        let (_, rows) = draw(&mut state);
        assert_eq!(rows, None, "closed: the owner draws the cut row");
        assert!(state.available());
        state.on_key(&key(KeyCode::Right));
        let (buf, rows) = draw(&mut state);
        let rows: Vec<String> = rows
            .expect("open: inline rows")
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert_eq!(rows.join(" "), FULL);
        assert!(rows.len() > 1, "wrapped to the row's width");
        assert!(
            (0..6).all(|y| !row_text(&buf, y).contains("redesign")),
            "no tooltip while the row wraps in place"
        );
        // the tooltip mode never returns inline rows
        let mut tip = RevealState::default();
        let mut got = Some(Vec::new());
        frame(80, 6, Interaction::default(), |_, ctx| {
            tip.begin(ctx);
            tip.open = true;
            got = tip.inline(ctx, id, 20, FULL, true, 4);
        });
        assert_eq!(got, None);
        assert!(tip.available());
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
    fn the_reveal_hint_and_more_lines_take_the_theme_strings() {
        let theme = Theme::junie().with_strings(crate::theme::Strings {
            reveal_open: "Ganzer Text",
            reveal_close: "Zuklappen",
            more_count: "{} weitere",
            ..crate::theme::Strings::DEFAULT
        });
        let id = WidgetId::of("row");
        let area = Rect::new(4, 2, 20, 1);
        let mut state = RevealState::default();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 6));
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        state.begin(&ctx);
        cut_cell(&mut buf, area);
        state.cell(&mut ctx, id, area, FULL, true);
        assert_eq!(state.hint(), Some(("→", "Ganzer Text")));
        state.on_key(&key(KeyCode::Right));
        assert_eq!(state.hint(), Some(("←", "Zuklappen")));
        let lines: Vec<String> = (0..10).map(|i| format!("l{i}")).collect();
        let (top, _) = window(&lines, 4, 0, &theme);
        assert_eq!(top[3].text, "↓ 7 weitere");
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
