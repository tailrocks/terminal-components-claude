//! Contextual key hints for a footer line: `key Action` pairs, dropping the
//! least important from the right when the line is narrow.
//!
//! # Hint vocabulary
//!
//! Key labels: `↑↓`, `←→`, `↑↓←→` (no spaces), ranges with an en dash
//! (`1–6`), `Delete` in full. [`normalize_key`] enforces this at render.
//!
//! Actions start with a verb, and an action that opens a dialog or a
//! further step ends with `…`. Toggles name the switch, not the state
//! ([`toggle_hint`]). A focused button names itself (`Button::hint`), and a
//! dialog builds its own row (`Dialog::hints`).
//!
//! Esc: `Esc Cancel` leaves an edit or a modal without saving, `Esc Close`
//! closes an overlay or menu, `Esc Back` returns to the previous page.
//! Field edits use [`EDIT_FIELD`]: `Enter Apply · Esc Cancel · Tab Next
//! field`.
//!
//! Within one app a letter keeps one meaning across screens:
//!
//! | Key      | Meaning                                         | Apps          |
//! |----------|-------------------------------------------------|---------------|
//! | `d`      | delete or remove (`…` when it asks first)       | all           |
//! | `x`      | close or stop what is in front (pane, launch)   | jackin        |
//! | `t`      | turn a thing on or off (account, debug, stop)   | jackin        |
//! | `c`      | open Accounts                                   | jackin        |
//! | `v`      | switch the view (unified / side by side)        | jackin        |
//! | `!`      | open a shell                                    | jackin        |
//! | `Ctrl+S` | save or submit what the screen is about         | all           |
//! | `Ctrl+D` | switch Data / Structure                         | TablePro      |
//! | `Alt+D`  | duplicate (a row, a connection)                 | TablePro      |
//!
//! Pane commands behind a prefix (`Ctrl+B c`, `Ctrl+B d`, `Ctrl+B x`) follow
//! tmux, the convention their users already know, and are a separate
//! layer: the prefix names the namespace.

use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::theme::{BadgeKind, HintAlign, HintBarStyle, Theme, Tone};

/// The key-label vocabulary. Every hint names keys the same way: arrow
/// pairs without a space, ranges with an en dash, `Delete` in full.
pub const UP_DOWN: &str = "↑↓";
pub const LEFT_RIGHT: &str = "←→";
pub const ARROWS: &str = "↑↓←→";
pub const DELETE: &str = "Delete";

/// A key range such as `1–6`.
pub fn range(from: u32, to: u32) -> String {
    format!("{from}–{to}")
}

/// Normalise a key label to the vocabulary: spaces between arrow glyphs
/// are dropped (`↑ ↓` → `↑↓`), a hyphen between digits becomes an en dash
/// (`1-6` → `1–6`) and `Del` is spelled `Delete`. Rendering applies this,
/// so a stray spelling cannot reach the screen.
pub fn normalize_key(key: &str) -> Cow<'_, str> {
    const ARROW: [char; 4] = ['↑', '↓', '←', '→'];
    if key == "Del" {
        return Cow::Borrowed(DELETE);
    }
    let chars: Vec<char> = key.chars().collect();
    let mut out = String::with_capacity(key.len());
    let mut changed = false;
    for (i, &c) in chars.iter().enumerate() {
        let prev = i.checked_sub(1).map(|j| chars[j]);
        let next = chars.get(i + 1).copied();
        if c == ' '
            && prev.is_some_and(|p| ARROW.contains(&p))
            && next.is_some_and(|n| ARROW.contains(&n))
        {
            changed = true;
            continue;
        }
        if c == '-'
            && prev.is_some_and(|p| p.is_ascii_digit())
            && next.is_some_and(|n| n.is_ascii_digit())
        {
            out.push('–');
            changed = true;
            continue;
        }
        out.push(c);
    }
    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(key)
    }
}

/// The hint for a two-state toggle. A toggle hint starts with a verb and
/// names what pressing the key does from the current state, so it never
/// reads like the state itself: `Ctrl+H Show hidden` while hidden files
/// are hidden, `Ctrl+H Hide hidden` while they are shown.
/// `when_on` is the action offered while `on` is true.
pub const fn toggle_hint(
    key: &'static str,
    on: bool,
    when_on: &'static str,
    when_off: &'static str,
) -> Hint {
    hint(key, if on { when_on } else { when_off })
}

/// The hints for an inline field being edited, in every app: Enter
/// applies the edit, Esc cancels it, Tab applies and moves on. "Commit"
/// is avoided because it collides with a database commit.
pub const EDIT_FIELD: [Hint; 3] = [
    hint("Enter", "Apply"),
    hint("Esc", "Cancel"),
    hint("Tab", "Next field"),
];

/// The hint for the keyboard context menu ([`crate::ui::context::ContextKeys`]
/// default): `Shift+F10 Menu`. A screen whose focused widget has a context
/// menu lists it last, so a narrow bar drops it first.
pub const CONTEXT_MENU: Hint = hint("Shift+F10", "Menu");

/// [`EDIT_FIELD`] for a grid cell: Tab moves to the next cell.
pub const EDIT_CELL: [Hint; 3] = [
    hint("Enter", "Apply"),
    hint("Esc", "Cancel"),
    hint("Tab", "Next cell"),
];

/// One `key Action` pair. The action is usually a literal, but a component
/// that knows its own state (a dialog naming its focused button) builds it
/// at runtime, so it is a [`Cow`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub key: &'static str,
    pub action: Cow<'static, str>,
    /// Whether the hint survives a narrow row. Default
    /// [`HintPriority::Auto`]: pinned when its key is one of the theme's
    /// [`HintBarStyle::pinned_keys`] (Esc, ?, F1).
    pub priority: HintPriority,
}

/// How a hint competes for room in a narrow footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HintPriority {
    /// Pinned when the key is one of the theme's pinned keys, otherwise
    /// normal.
    #[default]
    Auto,
    /// Dropped from the right when the row is too narrow.
    Normal,
    /// Kept while any normal hint can still be dropped.
    Pinned,
}

impl Hint {
    /// A hint whose action is computed (owned) or borrowed.
    pub fn new(key: &'static str, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key,
            action: action.into(),
            priority: HintPriority::Auto,
        }
    }

    /// Set how the hint competes for room.
    pub fn priority(mut self, priority: HintPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Never drop this hint while a normal one can go.
    pub fn pinned(self) -> Self {
        self.priority(HintPriority::Pinned)
    }

    /// Whether the hint is pinned under `style`.
    pub fn is_pinned(&self, style: &HintBarStyle) -> bool {
        match self.priority {
            HintPriority::Pinned => true,
            HintPriority::Normal => false,
            HintPriority::Auto => style.pinned_keys.contains(&self.key),
        }
    }
}

pub const fn hint(key: &'static str, action: &'static str) -> Hint {
    Hint {
        key,
        action: Cow::Borrowed(action),
        priority: HintPriority::Auto,
    }
}

/// Render hints left-aligned; `badge` (e.g. EDIT) leads; `right` is a
/// status message that always wins the right edge. Returns the number of
/// hints that fit.
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<&str>,
) -> usize {
    render_toned(
        area,
        buf,
        t,
        hints,
        badge,
        right.map(|r| (r, Tone::Secondary)),
    )
}

/// Like [`render`], with a toned status: an error status is drawn as
/// `! message` in the error tone and a warning as `▲ message` in the
/// warning tone, so the weight survives monochrome. Layout follows the
/// theme's [`HintBarStyle`]: hints that do not fit are first shown as keys
/// only, then dropped from the right (pinned ones last), and a faint `…`
/// marks the cut.
pub fn render_toned(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
) -> usize {
    render_styled(area, buf, t, hints, badge, right, &t.hintbar)
}

/// Like [`render_toned`], with the theme's style but an explicit alignment.
pub fn render_aligned(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
    align: HintAlign,
) -> usize {
    let style = HintBarStyle { align, ..t.hintbar };
    render_styled(area, buf, t, hints, badge, right, &style)
}

/// Gap after each hint, and before the cut marker.
const GAP: u16 = 2;

/// Cells one hint takes, without the gap after it. A compact hint shows
/// its key alone.
fn hint_width(h: &Hint, compact: bool) -> u16 {
    let kw = crate::ui::text::width(&normalize_key(h.key)) as u16;
    if compact {
        kw
    } else {
        kw + 1 + crate::ui::text::width(&h.action) as u16
    }
}

/// Cells the hints at `keep` take, each followed by a gap, plus a gap for
/// the cut marker when some are left out.
fn cost(hints: &[Hint], keep: &[usize], compact: &[bool]) -> u32 {
    let mut c: u32 = keep
        .iter()
        .map(|&i| (hint_width(&hints[i], compact[i]) + GAP) as u32)
        .sum();
    if keep.len() < hints.len() {
        c += GAP as u32;
    }
    c
}

/// Which hints are drawn, and which of them compactly, so they fit in
/// `room` cells: all in full; else all with normal hints as keys only
/// (pinned hints keep their action, they are the way out); else in full
/// with normal hints dropped from the right, then pinned ones.
fn fit(hints: &[Hint], room: u16, style: &HintBarStyle) -> (Vec<usize>, Vec<bool>) {
    let room = room as u32;
    let all: Vec<usize> = (0..hints.len()).collect();
    let full = vec![false; hints.len()];
    if cost(hints, &all, &full) <= room {
        return (all, full);
    }
    let pinned: Vec<bool> = hints.iter().map(|h| h.is_pinned(style)).collect();
    if style.compact {
        let compact: Vec<bool> = pinned.iter().map(|p| !p).collect();
        if cost(hints, &all, &compact) <= room {
            return (all, compact);
        }
    }
    let mut keep = all;
    let mut order: Vec<usize> = (0..hints.len()).rev().filter(|&i| !pinned[i]).collect();
    order.extend((0..hints.len()).rev().filter(|&i| pinned[i]));
    for gone in order {
        if cost(hints, &keep, &full) <= room {
            break;
        }
        keep.retain(|&i| i != gone);
    }
    (keep, full)
}

/// [`render_toned`] with an explicit [`HintBarStyle`].
pub fn render_styled(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
    style: &HintBarStyle,
) -> usize {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return 0;
    }
    let mut x = area.x + 1;
    if let Some((text, kind)) = badge {
        let b =
            crate::ui::text::truncate(&format!(" {text} "), area.width.saturating_sub(2) as usize);
        if !b.is_empty() {
            buf.set_string(x, area.y, &b, t.badge(kind));
            x = (x + crate::ui::text::width(&b) as u16 + 2).min(area.right());
        }
    }
    let mut right_w = 0u16;
    if let Some((r, tone)) = right {
        // the glyph carries the meaning in monochrome, the tone in colour
        let mark = match tone {
            Tone::Error => "!",
            Tone::Warning => "▲",
            _ => "",
        };
        let text = if mark.is_empty() {
            r.to_owned()
        } else {
            format!("{mark} {r}")
        };
        let mut available = area.right().saturating_sub(x + 1) as usize;
        if !hints.is_empty() {
            // the pinned hints (the way out, help) keep their room; the
            // status still keeps its minimum share of the row
            let share = area.width as usize * style.status_min_share.min(100) as usize / 100;
            let pinned: Vec<usize> = (0..hints.len())
                .filter(|&i| hints[i].is_pinned(style))
                .collect();
            if !pinned.is_empty() {
                let need = cost(hints, &pinned, &vec![false; hints.len()]) as usize;
                available = available.min(share.max(available.saturating_sub(need + 2)));
            }
        }
        let text = crate::ui::text::truncate(&text, available);
        let w = crate::ui::text::width(&text) as u16;
        if w > 0 {
            let st = ratatui::style::Style::new().fg(t.tone(tone));
            buf.set_string(area.right() - w - 1, area.y, &text, st);
            if !mark.is_empty() {
                buf.set_string(
                    area.right() - w - 1,
                    area.y,
                    mark,
                    st.add_modifier(ratatui::style::Modifier::BOLD),
                );
            }
            right_w = w + 3;
        }
    }
    let limit = area.right().saturating_sub(right_w);
    let (keep, compact) = fit(hints, limit.saturating_sub(x), style);
    let cut = keep.len() < hints.len();
    if style.align == HintAlign::Center {
        // centre the drawn block itself: the gap after the last hint is
        // not part of it, the cut marker is
        let mut used = cost(hints, &keep, &compact) as u16;
        used = if cut {
            used.saturating_sub(GAP) + 1
        } else {
            used.saturating_sub(GAP)
        };
        let free = area.width.saturating_sub(used);
        let mid = area.x + free / 2;
        // never past the badge, never under the status
        x = mid.max(x).min(limit.saturating_sub(used).max(x));
    }
    for &i in &keep {
        let h = &hints[i];
        let key = normalize_key(h.key);
        let kw = crate::ui::text::width(&key) as u16;
        buf.set_string(x, area.y, &key, t.key_hint_key());
        if !compact[i] {
            buf.set_string(x + kw + 1, area.y, &h.action, t.key_hint_action());
        }
        x += hint_width(h, compact[i]) + GAP;
    }
    if cut && x < limit {
        buf.set_string(x, area.y, "…", t.faint());
    }
    keep.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row_text(buf: &Buffer, width: u16) -> String {
        (0..width)
            .map(|x| buf[(x, 0)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn warning_and_error_statuses_carry_glyphs_so_mono_keeps_the_weight() {
        let t = Theme::junie();
        let hints = [hint("Esc", "Back")];
        for (tone, mark) in [(Tone::Warning, "▲"), (Tone::Error, "!")] {
            let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
            render_toned(
                Rect::new(0, 0, 60, 1),
                &mut buf,
                &t,
                &hints,
                None,
                Some(("disk low", tone)),
            );
            let row = row_text(&buf, 60);
            assert!(row.contains(&format!("{mark} disk low")), "{row:?}");
        }
        let mut plain = Buffer::empty(Rect::new(0, 0, 60, 1));
        render_toned(
            Rect::new(0, 0, 60, 1),
            &mut plain,
            &t,
            &hints,
            None,
            Some(("Saved", Tone::Secondary)),
        );
        let row = row_text(&plain, 60);
        assert!(row.contains("Saved"), "{row:?}");
        assert!(!row.contains('▲') && !row.contains('!'), "{row:?}");
    }

    #[test]
    fn long_status_keeps_its_severity_and_yields_hints_without_overlapping_edit_badge() {
        for level in [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Mono,
        ] {
            let t = Theme::for_level(level);
            let mut buf = Buffer::empty(Rect::new(0, 0, 32, 1));
            let count = render_toned(
                *buf.area(),
                &mut buf,
                &t,
                &[hint("Esc", "Cancel")],
                Some(("EDIT", BadgeKind::Edit)),
                Some(("无法保存配置文件：目标目录只读 e\u{301}", Tone::Error)),
            );
            let row = row_text(&buf, 32);
            assert_eq!(count, 0);
            assert!(row.starts_with("  EDIT "), "{row:?}");
            assert!(row.contains("! 无"), "{row:?}");
            assert!(row.contains('…'), "{row:?}");
            assert!(!row.contains("Cancel"));
        }
    }

    #[test]
    fn narrow_status_and_badge_stay_inside_assigned_area() {
        let t = Theme::junie();
        for width in 1..16 {
            let area = Rect::new(2, 1, width, 1);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
            for cell in &mut buf.content {
                cell.set_symbol(".");
            }
            render_toned(
                area,
                &mut buf,
                &t,
                &[hint("Esc", "Cancel")],
                Some(("EDIT", BadgeKind::Edit)),
                Some(("long failure message", Tone::Error)),
            );
            for y in 0..3 {
                for x in 0..20 {
                    if !area.contains((x, y).into()) {
                        assert_eq!(buf[(x, y)].symbol(), ".");
                    }
                }
            }
        }
    }

    #[test]
    fn key_labels_render_in_one_vocabulary() {
        assert_eq!(normalize_key("↑ ↓"), UP_DOWN);
        assert_eq!(normalize_key("← →"), LEFT_RIGHT);
        assert_eq!(normalize_key("↑ ↓ ← →"), ARROWS);
        assert_eq!(normalize_key("1-6"), range(1, 6));
        assert_eq!(normalize_key("Del"), DELETE);
        // labels that are already right, or merely contain spaces, stay as they are
        assert!(matches!(
            normalize_key("PgUp PgDn"),
            Cow::Borrowed("PgUp PgDn")
        ));
        assert!(matches!(normalize_key("Ctrl+-"), Cow::Borrowed("Ctrl+-")));
        assert!(matches!(normalize_key("Shift+↑"), Cow::Borrowed(_)));

        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        render(
            Rect::new(0, 0, 40, 1),
            &mut buf,
            &t,
            &[hint("↑ ↓", "Move"), hint("1-6", "Jump")],
            None,
            None,
        );
        let row = row_text(&buf, 40);
        assert!(row.contains("↑↓ Move"), "{row}");
        assert!(row.contains("1–6 Jump"), "{row}");
    }

    #[test]
    fn a_toggle_hint_offers_the_switch_not_the_state() {
        let shown = toggle_hint("Ctrl+H", true, "Hide hidden", "Show hidden");
        assert_eq!(shown.action, "Hide hidden");
        let hidden = toggle_hint("Ctrl+H", false, "Hide hidden", "Show hidden");
        assert_eq!(hidden.action, "Show hidden");
    }

    fn draw(width: u16, t: &Theme, hints: &[Hint], status: Option<(&str, Tone)>) -> String {
        let mut buf = Buffer::empty(Rect::new(0, 0, width, 1));
        render_toned(Rect::new(0, 0, width, 1), &mut buf, t, hints, None, status);
        row_text(&buf, width)
    }

    #[test]
    fn the_way_out_and_help_are_pinned_while_other_hints_drop() {
        let t = Theme::junie();
        let hints = [
            hint("Enter", "Open"),
            hint("Space", "Choose"),
            hint("u", "Add URL"),
            hint("Tab", "Next"),
            hint("?", "Help"),
            hint("Esc", "Cancel"),
        ];
        let row = draw(36, &t, &hints, None);
        assert!(row.contains("Enter Open"), "{row:?}");
        assert!(row.contains("? Help"), "{row:?}");
        assert!(row.contains("Esc Cancel"), "{row:?}");
        assert!(!row.contains("Next") && row.contains('…'), "{row:?}");
        // the order stays the caller's
        assert!(row.find("Help") < row.find("Cancel"), "{row:?}");
        // pinning is per hint too, and Auto follows the theme's keys
        let row = draw(
            36,
            &t,
            &[
                hint("Enter", "Open"),
                hint("Tab", "Next").pinned(),
                hint("x", "Close a long tab name"),
            ],
            None,
        );
        assert!(
            row.contains("Tab Next") && !row.contains("Close"),
            "{row:?}"
        );
        let row = draw(
            24,
            &t,
            &[
                hint("Enter", "Open the thing now"),
                hint("Esc", "Back").priority(HintPriority::Normal),
            ],
            None,
        );
        assert!(!row.contains("Back"), "{row:?}");
    }

    #[test]
    fn a_long_status_leaves_the_pinned_hints_their_room() {
        let t = Theme::junie();
        let hints = [
            hint("Enter", "Save"),
            hint("Esc", "Back"),
            hint("?", "Help"),
        ];
        let status = "Saved the preview of every changed setting to the profile on disk";
        let row = draw(72, &t, &hints, Some((status, Tone::Secondary)));
        assert!(row.contains("Esc Back"), "{row:?}");
        assert!(row.contains("? Help"), "{row:?}");
        assert!(row.contains("Saved the preview"), "{row:?}");
        // but the status keeps its minimum share of the row
        let mut narrow = t;
        narrow.hintbar.status_min_share = 90;
        let row = draw(72, &narrow, &hints, Some((status, Tone::Secondary)));
        assert!(!row.contains("Esc Back"), "{row:?}");
        // a short status is never cut for hints
        let row = draw(40, &t, &hints, Some(("Saved", Tone::Secondary)));
        assert!(row.contains("Saved"), "{row:?}");
    }

    #[test]
    fn compact_mode_shows_keys_before_dropping_any() {
        let mut t = Theme::junie();
        let hints = [
            hint("Enter", "Open"),
            hint("Space", "Choose"),
            hint("u", "Add URL"),
            hint("Esc", "Cancel"),
        ];
        // off by default: hints drop from the right in full
        let row = draw(30, &t, &hints, None);
        assert!(
            row.contains("Enter Open") && !row.contains("URL"),
            "{row:?}"
        );
        t.hintbar.compact = true;
        let row = draw(30, &t, &hints, None);
        assert!(row.contains("Enter  Space  u  Esc Cancel"), "{row:?}");
        assert!(!row.contains('…'), "{row:?}");
    }

    #[test]
    fn a_centred_row_is_centred_on_its_text_not_its_trailing_gap() {
        let mut t = Theme::junie();
        t.hintbar.align = HintAlign::Center;
        // "Esc Back" is 8 cells: 32 free in 40, 16 a side
        let row = draw(40, &t, &[hint("Esc", "Back")], None);
        assert_eq!(row.find("Esc"), Some(16), "{row:?}");
        assert_eq!(row.len() - row.trim_end().len(), 16, "{row:?}");
    }
}
