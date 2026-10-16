//! Tooltips: a short description of a control, and its keyboard shortcut,
//! shown when the pointer rests on the control.
//!
//! Buttons, icons, toolbar items and status segments say what they do on
//! hover, the way macOS help tags do, so a shortcut can be discovered with
//! the mouse (until now only menus showed shortcuts). The tooltip follows
//! WCAG 2.2 SC 1.4.13 (content on hover or focus):
//!
//! - **Dismissible.** Esc hides it without moving the pointer or focus,
//!   and without reaching anything else (a dialog under it stays open).
//!   It stays hidden until the pointer leaves the control.
//! - **Hoverable.** The pointer can move onto the tooltip without it
//!   vanishing; its cells are a hover-only region, so a click there still
//!   reaches whatever is beneath.
//! - **Persistent.** It stays until the pointer leaves both the control and
//!   the tooltip, Esc dismisses it, or the control is pressed.
//!
//! It shows after the pointer has rested [`Tooltips::delay_ms`] (default
//! 600 ms, macOS-like) on the control, measured by
//! [`Interaction::hover_ms`](crate::ui::ctx::Interaction::hover_ms); moving
//! onto the tooltip itself keeps it at once. It draws through the overlay
//! layer, below the control (above when there is no room), and only one
//! tooltip shows at a time. Keyboard use hides it (hover is suppressed
//! until the pointer moves).
//!
//! Any widget shows one with [`show`]; [`Button::tooltip`] and
//! [`Button::shortcut`] wire it for buttons. The policy is app-wide on
//! [`RenderCtx::tooltips`], with a per-call override.
//!
//! [`Button::tooltip`]: crate::widgets::button::Button::tooltip
//! [`Button::shortcut`]: crate::widgets::button::Button::shortcut

use std::cell::Cell;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::event::{Input, Outcome};
use crate::core::id::WidgetId;
use crate::theme::ColorLevel;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{self, Placement};
use crate::ui::text::{truncate, width, wrap};

/// The tooltip policy: app-wide on [`RenderCtx::tooltips`], overridable
/// per call of [`show`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tooltips {
    /// Tooltips show at all. Default on.
    pub enabled: bool,
    /// How long the pointer rests on a control before its tooltip shows,
    /// in ms. Default 600.
    pub delay_ms: u64,
    /// Widest a tooltip's text runs before it wraps, in cells. Default 48.
    pub max_width: u16,
}

impl Tooltips {
    /// No tooltips.
    pub const OFF: Self = Self {
        enabled: false,
        delay_ms: 600,
        max_width: 48,
    };

    pub const fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
        self
    }

    pub const fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    pub const fn max_width(mut self, cells: u16) -> Self {
        self.max_width = cells;
        self
    }
}

impl Default for Tooltips {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_ms: 600,
            max_width: 48,
        }
    }
}

/// What a control's tooltip says: a description and, optionally, the key
/// that does the same (drawn muted after it, through
/// [`Glyphs::key_label`](crate::theme::Glyphs::key_label)).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tip {
    pub text: String,
    pub shortcut: Option<String>,
}

impl Tip {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            shortcut: None,
        }
    }

    pub fn shortcut(mut self, key: &str) -> Self {
        self.shortcut = Some(key.to_owned());
        self
    }
}

/// The hover id of `owner`'s tooltip box.
pub fn tip_id(owner: WidgetId) -> WidgetId {
    owner.sub("junie.tooltip")
}

thread_local! {
    /// The control whose tooltip the last frame drew.
    static SHOWN: Cell<Option<WidgetId>> = const { Cell::new(None) };
    /// The control whose tooltip Esc (or a press) hid, until the pointer
    /// leaves it.
    static DISMISSED: Cell<Option<WidgetId>> = const { Cell::new(None) };
    /// This frame already queued a tooltip.
    static QUEUED: Cell<bool> = const { Cell::new(false) };
}

/// A frame starts ([`RenderCtx::new`]).
pub(crate) fn begin_frame() {
    QUEUED.with(|q| q.set(false));
    SHOWN.with(|s| s.set(None));
}

/// Show `tip` for the control `owner` drawn at `anchor`, when the pointer
/// rests on it (or on the tooltip), under `policy` or the app-wide
/// [`RenderCtx::tooltips`]. Call it from the control's render.
pub fn show(
    ctx: &mut RenderCtx,
    owner: WidgetId,
    anchor: Rect,
    tip: &Tip,
    policy: Option<Tooltips>,
) {
    let policy = policy.unwrap_or(ctx.tooltips);
    if !policy.enabled || ctx.inert || anchor.is_empty() || tip.text.is_empty() {
        return;
    }
    let live = ctx.interaction.live_hover();
    let id = tip_id(owner);
    let on_owner = live == Some(owner);
    let on_tip = live == Some(id);
    let dismissed = DISMISSED.with(Cell::get) == Some(owner);
    if !on_owner && !on_tip {
        if dismissed {
            DISMISSED.with(|d| d.set(None));
        }
        return;
    }
    if ctx.interaction.pressed == Some(owner) {
        // pressing the control hides its tooltip, as on macOS
        DISMISSED.with(|d| d.set(Some(owner)));
        return;
    }
    if dismissed || QUEUED.with(Cell::get) {
        return;
    }
    // resting on the tooltip itself keeps it: no second wait
    if on_owner && !crate::ui::reveal::hover_ready(ctx, policy.delay_ms) {
        return;
    }
    QUEUED.with(|q| q.set(true));
    let tip = tip.clone();
    ctx.overlay(move |buf, ctx| draw(buf, ctx, owner, anchor, &tip, policy));
}

/// Esc while a tooltip shows dismisses it (and goes no further); `None`
/// for every other input, which the app handles as usual.
/// [`crate::runtime`] calls this before dispatching each input.
pub fn on_input(input: &Input) -> Option<Outcome> {
    let Input::Key(k) = input else {
        return None;
    };
    if k.code != KeyCode::Esc || !k.mods.is_empty() {
        return None;
    }
    let owner = SHOWN.with(Cell::get)?;
    DISMISSED.with(|d| d.set(Some(owner)));
    SHOWN.with(|s| s.set(None));
    Some(Outcome::Changed)
}

/// Where `tip` for `anchor` lands on `screen`, and its rows.
fn layout(
    screen: Rect,
    anchor: Rect,
    tip: &Tip,
    policy: Tooltips,
    key: Option<&str>,
) -> (Rect, Vec<String>) {
    let room = (screen.width.saturating_sub(2))
        .min(policy.max_width)
        .max(1) as usize;
    let mut lines = wrap(&tip.text, room);
    if let Some(key) = key {
        // the shortcut follows the last line, or takes its own
        let last = lines.last().map(|l| width(l)).unwrap_or(0);
        if last + 2 + width(key) <= room {
            if let Some(l) = lines.last_mut() {
                l.push_str("  ");
            }
        } else {
            lines.push(String::new());
        }
    }
    let key_w = key.map_or(0, width);
    let w = lines.iter().map(|l| width(l)).max().unwrap_or(0) as u16;
    let w = w.max(lines.last().map_or(0, |l| width(l) + key_w) as u16) + 2;
    let h = lines.len() as u16;
    let area = popup::place_with(screen, anchor, w, h, Placement::Below, popup::Place::FLUSH);
    (area, lines)
}

fn draw(
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    owner: WidgetId,
    anchor: Rect,
    tip: &Tip,
    policy: Tooltips,
) {
    let screen = *buf.area();
    let t = ctx.theme;
    let key = tip
        .shortcut
        .as_deref()
        .map(|k| t.glyphs.key_label(k).into_owned());
    let (area, lines) = layout(screen, anchor, tip, policy, key.as_deref());
    let area = area.intersection(screen);
    if area.is_empty() {
        return;
    }
    SHOWN.with(|s| s.set(Some(owner)));
    // the popover plane, as the expansion tooltip; over a popover it steps
    // down a plane so it reads as its own layer
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
    let inner = area.width.saturating_sub(2) as usize;
    let rows = area.height as usize;
    for (k, line) in lines.iter().take(rows).enumerate() {
        let y = area.y + k as u16;
        buf.set_string(area.x + 1, y, truncate(line, inner), st);
        if k + 1 == lines.len()
            && let Some(key) = &key
        {
            let x = area.x + 1 + width(line) as u16;
            let left = (area.right().saturating_sub(1)).saturating_sub(x) as usize;
            buf.set_string(x, y, truncate(key, left), st.fg(t.text_muted));
        }
    }
    // hoverable: the pointer may rest on the tooltip; clicks pass beneath
    ctx.hits.register_hover(tip_id(owner), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Key;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;
    use ratatui::layout::Position;

    const OWNER: WidgetId = WidgetId::of("save");
    const ANCHOR: Rect = Rect::new(2, 1, 8, 1);

    fn tip() -> Tip {
        Tip::new("Save the query").shortcut("Ctrl+S")
    }

    fn frame(interaction: Interaction) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 8));
        {
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            ctx.clickable(OWNER, ANCHOR);
            show(&mut ctx, OWNER, ANCHOR, &tip(), None);
            ctx.flush_overlays(&mut buf);
        }
        (buf, hits)
    }

    fn text(buf: &Buffer) -> String {
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    fn hovering(id: WidgetId, ms: Option<u64>) -> Interaction {
        Interaction {
            hover: Some(id),
            hover_ms: ms,
            ..Default::default()
        }
    }

    fn esc() -> Input {
        Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        })
    }

    /// The tooltip waits for the pointer to rest, then shows the
    /// description and the shortcut below the control.
    #[test]
    fn a_resting_pointer_shows_the_description_and_shortcut() {
        let (buf, _) = frame(hovering(OWNER, Some(100)));
        assert!(!text(&buf).contains("Save the query"), "not yet");
        assert_eq!(
            crate::ui::reveal::hover_wait().map(|w| w.1),
            Some(500),
            "the runtime is told when to draw again"
        );
        let (buf, hits) = frame(hovering(OWNER, Some(600)));
        let s = text(&buf);
        let row: Vec<&str> = s.lines().collect();
        assert!(row[2].contains("Save the query  Ctrl+S"), "{s}");
        // muted shortcut
        let x = row[2].find("Ctrl").unwrap() as u16;
        assert_eq!(buf[(x, 2)].fg, Theme::junie().text_muted);
        // hoverable, but clicks pass beneath
        assert_eq!(hits.hover_at(Position::new(4, 2)), Some(tip_id(OWNER)));
        assert_eq!(hits.hit(Position::new(4, 2)), None);
        // not hovered: nothing
        let (buf, _) = frame(Interaction::default());
        assert!(!text(&buf).contains("Save the query"));
    }

    /// WCAG 1.4.13: the pointer can move onto the tooltip and it stays;
    /// Esc dismisses it until the pointer leaves; a press hides it.
    #[test]
    fn it_is_hoverable_dismissible_and_persistent() {
        let (buf, _) = frame(hovering(OWNER, Some(700)));
        assert!(text(&buf).contains("Save the query"));
        // hoverable: on the tooltip itself it stays, with no new wait
        let (buf, _) = frame(hovering(tip_id(OWNER), Some(0)));
        assert!(text(&buf).contains("Save the query"));
        // dismissible: Esc hides it and is taken
        assert_eq!(on_input(&esc()), Some(Outcome::Changed));
        let (buf, _) = frame(hovering(OWNER, Some(5_000)));
        assert!(!text(&buf).contains("Save the query"), "stays dismissed");
        assert_eq!(on_input(&esc()), None, "nothing shown: Esc goes on");
        // leaving re-arms it
        frame(Interaction::default());
        let (buf, _) = frame(hovering(OWNER, Some(700)));
        assert!(text(&buf).contains("Save the query"));
        // a press hides it
        let (buf, _) = frame(Interaction {
            pressed: Some(OWNER),
            ..hovering(OWNER, Some(800))
        });
        assert!(!text(&buf).contains("Save the query"));
        frame(Interaction::default());
        // keyboard use (suppressed hover) hides it
        let (buf, _) = frame(Interaction {
            hover_suppressed: true,
            ..hovering(OWNER, Some(800))
        });
        assert!(!text(&buf).contains("Save the query"));
    }

    #[test]
    fn the_policy_turns_tooltips_off_or_changes_the_delay() {
        let theme = Theme::junie();
        let run = |policy: Tooltips, ms: u64| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 8));
            let mut ctx = RenderCtx::new(&theme, hovering(OWNER, Some(ms)), &mut hits, &mut ring);
            ctx.tooltips = policy;
            show(&mut ctx, OWNER, ANCHOR, &tip(), None);
            ctx.flush_overlays(&mut buf);
            text(&buf).contains("Save the query")
        };
        assert!(!run(Tooltips::OFF, 5_000));
        assert!(run(Tooltips::default().delay_ms(0), 0));
        assert!(!run(Tooltips::default(), 599));
    }
}
