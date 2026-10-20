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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tooltips {
    pub enabled: bool,

    pub delay_ms: u64,

    pub max_width: u16,
}

impl Tooltips {
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

pub fn tip_id(owner: WidgetId) -> WidgetId {
    owner.sub("junie.tooltip")
}

thread_local! {

    static SHOWN: Cell<Option<WidgetId>> = const { Cell::new(None) };


    static DISMISSED: Cell<Option<WidgetId>> = const { Cell::new(None) };

    static QUEUED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn begin_frame() {
    QUEUED.with(|q| q.set(false));
    SHOWN.with(|s| s.set(None));
}

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
        DISMISSED.with(|d| d.set(Some(owner)));
        return;
    }
    if dismissed || QUEUED.with(Cell::get) {
        return;
    }

    if on_owner && !crate::ui::reveal::hover_ready(ctx, policy.delay_ms) {
        return;
    }
    QUEUED.with(|q| q.set(true));
    let tip = tip.clone();
    ctx.overlay(move |buf, ctx| draw(buf, ctx, owner, anchor, &tip, policy));
}

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

        let x = row[2].find("Ctrl").unwrap() as u16;
        assert_eq!(buf[(x, 2)].fg, Theme::junie().text_muted);

        assert_eq!(hits.hover_at(Position::new(4, 2)), Some(tip_id(OWNER)));
        assert_eq!(hits.hit(Position::new(4, 2)), None);

        let (buf, _) = frame(Interaction::default());
        assert!(!text(&buf).contains("Save the query"));
    }

    #[test]
    fn it_is_hoverable_dismissible_and_persistent() {
        let (buf, _) = frame(hovering(OWNER, Some(700)));
        assert!(text(&buf).contains("Save the query"));

        let (buf, _) = frame(hovering(tip_id(OWNER), Some(0)));
        assert!(text(&buf).contains("Save the query"));

        assert_eq!(on_input(&esc()), Some(Outcome::Changed));
        let (buf, _) = frame(hovering(OWNER, Some(5_000)));
        assert!(!text(&buf).contains("Save the query"), "stays dismissed");
        assert_eq!(on_input(&esc()), None, "nothing shown: Esc goes on");

        frame(Interaction::default());
        let (buf, _) = frame(hovering(OWNER, Some(700)));
        assert!(text(&buf).contains("Save the query"));

        let (buf, _) = frame(Interaction {
            pressed: Some(OWNER),
            ..hovering(OWNER, Some(800))
        });
        assert!(!text(&buf).contains("Save the query"));
        frame(Interaction::default());

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
