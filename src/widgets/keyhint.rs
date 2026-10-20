use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::core::event::ClientPlatform;

use crate::theme::{BadgeKind, HintAlign, HintBarStyle, StatusKind, Theme, Tone};

pub const UP_DOWN: &str = "↑↓";
pub const LEFT_RIGHT: &str = "←→";
pub const ARROWS: &str = "↑↓←→";
pub const DELETE: &str = "Delete";

pub fn range(from: u32, to: u32) -> String {
    format!("{from}–{to}")
}

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

pub fn modifier_click(mods: KeyModifiers, platform: ClientPlatform) -> Option<&'static str> {
    const MAC: [(KeyModifiers, &str); 3] = [
        (KeyModifiers::ALT, "Option+click"),
        (KeyModifiers::SUPER, "Cmd+click"),
        (KeyModifiers::CONTROL, "Ctrl+click"),
    ];
    const OTHER: [(KeyModifiers, &str); 3] = [
        (KeyModifiers::CONTROL, "Ctrl+click"),
        (KeyModifiers::ALT, "Alt+click"),
        (KeyModifiers::SUPER, "Super+click"),
    ];
    let order = if platform.is_macos() { &MAC } else { &OTHER };
    order
        .iter()
        .find(|(m, _)| mods.contains(*m))
        .map(|&(_, label)| label)
}

pub const fn toggle_hint(
    key: &'static str,
    on: bool,
    when_on: &'static str,
    when_off: &'static str,
) -> Hint {
    hint(key, if on { when_on } else { when_off })
}

pub const EDIT_FIELD: [Hint; 3] = [
    hint("Enter", "Apply"),
    hint("Esc", "Cancel"),
    hint("Tab", "Next field"),
];

pub const CONTEXT_MENU: Hint = hint("Shift+F10", "Menu");

pub const EDIT_TEXT: [Hint; 2] = [hint("Tab", "Next field"), hint("Esc", "Done")];

pub const EDIT_CODE: [Hint; 3] = [
    hint("Tab", "Indent"),
    hint("F6", "Leave"),
    hint("Esc", "Done"),
];

pub const EDIT_CELL: [Hint; 3] = [
    hint("Enter", "Apply"),
    hint("Esc", "Cancel"),
    hint("Tab", "Next cell"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub key: &'static str,
    pub action: Cow<'static, str>,

    pub priority: HintPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HintPriority {
    #[default]
    Auto,

    Normal,

    Pinned,
}

impl Hint {
    pub fn new(key: &'static str, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key,
            action: action.into(),
            priority: HintPriority::Auto,
        }
    }

    pub fn priority(mut self, priority: HintPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn pinned(self) -> Self {
        self.priority(HintPriority::Pinned)
    }

    pub fn is_pinned(&self, style: &HintBarStyle) -> bool {
        match self.priority {
            HintPriority::Pinned => true,
            HintPriority::Normal => false,
            HintPriority::Auto => style.pinned_keys.contains(&self.key),
        }
    }
}

pub fn deliverable_line(line: &str) -> std::borrow::Cow<'_, str> {
    let keep = |part: &&str| {
        crate::core::keyboard::label_deliverable(part.split(' ').next().unwrap_or_default())
    };
    if line.split(" · ").all(|p| keep(&p)) {
        return std::borrow::Cow::Borrowed(line);
    }
    std::borrow::Cow::Owned(
        line.split(" · ")
            .filter(keep)
            .collect::<Vec<_>>()
            .join(" · "),
    )
}

pub const fn hint(key: &'static str, action: &'static str) -> Hint {
    Hint {
        key,
        action: Cow::Borrowed(action),
        priority: HintPriority::Auto,
    }
}

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

const GAP: u16 = 2;

fn hint_width(h: &Hint, compact: bool) -> u16 {
    let kw = crate::ui::text::width(&normalize_key(h.key)) as u16;
    if compact {
        kw
    } else {
        kw + 1 + crate::ui::text::width(&h.action) as u16
    }
}

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

pub fn width(hints: &[Hint], badge: Option<&str>, status: Option<&str>) -> u16 {
    let all: Vec<usize> = (0..hints.len()).collect();
    let full = vec![false; hints.len()];
    let badge = badge.map_or(0, |b| crate::ui::text::width(b) as u16 + 4);
    let status = status
        .map(|s| crate::ui::text::width(s) as u16)
        .filter(|w| *w > 0)
        .map_or(0, |w| w + 3);
    1 + badge + cost(hints, &all, &full) as u16 + status
}

pub fn render_status(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Option<StatusKind>)>,
) -> usize {
    render_status_at(area, buf, t, hints, badge, right).0
}

pub fn render_status_at(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Option<StatusKind>)>,
) -> (usize, Vec<(usize, Rect)>) {
    let right = right.map(|(text, kind)| match kind {
        Some(kind) => {
            let (mark, mark_style) = t.status(kind);
            Right {
                text,
                mark,
                mark_style,
                style: t.status_label(kind),
            }
        }
        None => Right::toned(t, text, Tone::Secondary),
    });
    let mut spots = Vec::new();
    let n = render_right(
        area,
        buf,
        t,
        hints,
        badge,
        right,
        &[],
        &t.hintbar,
        &mut spots,
    );
    (n, spots)
}

pub fn render_status_styled_at(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Option<StatusKind>)>,
    style: &HintBarStyle,
) -> (usize, Vec<(usize, Rect)>) {
    let right = right.map(|(text, kind)| match kind {
        Some(kind) => {
            let (mark, mark_style) = t.status(kind);
            Right {
                text,
                mark,
                mark_style,
                style: t.status_label(kind),
            }
        }
        None => Right::toned(t, text, Tone::Secondary),
    });
    let mut spots = Vec::new();
    let n = render_right(area, buf, t, hints, badge, right, &[], style, &mut spots);
    (n, spots)
}

struct Right<'a> {
    text: &'a str,
    mark: &'a str,
    mark_style: Style,
    style: Style,
}

impl<'a> Right<'a> {
    fn toned(t: &'a Theme, text: &'a str, tone: Tone) -> Self {
        let mark = match tone {
            Tone::Error => "!",
            Tone::Warning => t.glyphs.warning,
            _ => "",
        };
        let style = Style::new().fg(t.tone(tone));
        Self {
            text,
            mark,
            mark_style: style.add_modifier(ratatui::style::Modifier::BOLD),
            style,
        }
    }
}

pub fn render_styled(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
    style: &HintBarStyle,
) -> usize {
    let right = right.map(|(r, tone)| Right::toned(t, r, tone));
    render_right(
        area,
        buf,
        t,
        hints,
        badge,
        right,
        &[],
        style,
        &mut Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_parts(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
    parts: &[(String, Tone)],
    style: &HintBarStyle,
) -> usize {
    let right = right.map(|(r, tone)| Right::toned(t, r, tone));
    render_right(
        area,
        buf,
        t,
        hints,
        badge,
        right,
        parts,
        style,
        &mut Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_parts_at(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<(&str, Tone)>,
    parts: &[(String, Tone)],
    style: &HintBarStyle,
) -> (usize, Vec<(usize, Rect)>) {
    let right = right.map(|(r, tone)| Right::toned(t, r, tone));
    let mut spots = Vec::new();
    let n = render_right(area, buf, t, hints, badge, right, parts, style, &mut spots);
    (n, spots)
}

pub fn key_of(label: &str) -> Option<crate::core::event::Key> {
    use ratatui::crossterm::event::KeyCode;
    let label = normalize_key(label);
    let mut rest: &str = &label;
    let mut mods = KeyModifiers::NONE;
    loop {
        let (m, tail) = if let Some(t) = rest.strip_prefix("Ctrl+") {
            (KeyModifiers::CONTROL, t)
        } else if let Some(t) = rest.strip_prefix("Alt+") {
            (KeyModifiers::ALT, t)
        } else if let Some(t) = rest.strip_prefix("Shift+") {
            (KeyModifiers::SHIFT, t)
        } else {
            break;
        };
        if tail.is_empty() {
            break;
        }
        mods |= m;
        rest = tail;
    }
    let code = match rest {
        "Esc" => KeyCode::Esc,
        "Enter" | "↵" | "Return" => KeyCode::Enter,
        "Tab" if mods == KeyModifiers::SHIFT => {
            mods = KeyModifiers::NONE;
            KeyCode::BackTab
        }
        "Tab" => KeyCode::Tab,
        "Space" => KeyCode::Char(' '),
        "Delete" => KeyCode::Delete,
        "Backspace" => KeyCode::Backspace,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PgUp" | "PageUp" => KeyCode::PageUp,
        "PgDn" | "PageDown" => KeyCode::PageDown,
        "↑" => KeyCode::Up,
        "↓" => KeyCode::Down,
        "←" => KeyCode::Left,
        "→" => KeyCode::Right,
        _ => {
            if let Some(n) = rest.strip_prefix('F').and_then(|n| n.parse::<u8>().ok())
                && (1..=12).contains(&n)
            {
                KeyCode::F(n)
            } else {
                let mut chars = rest.chars();
                let c = chars.next()?;
                if chars.next().is_some() || c.is_whitespace() {
                    return None;
                }
                if mods.contains(KeyModifiers::CONTROL) {
                    KeyCode::Char(c.to_ascii_lowercase())
                } else {
                    if c.is_ascii_uppercase() {
                        mods |= KeyModifiers::SHIFT;
                    }
                    KeyCode::Char(c)
                }
            }
        }
    };
    Some(crate::core::event::Key { code, mods })
}

#[allow(clippy::too_many_arguments)]
fn render_right(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    hints: &[Hint],
    badge: Option<(&str, BadgeKind)>,
    right: Option<Right<'_>>,
    parts: &[(String, Tone)],
    style: &HintBarStyle,
    spots: &mut Vec<(usize, Rect)>,
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
    if let Some(Right {
        text: r,
        mark,
        mark_style,
        style: st,
    }) = right
    {
        let text = if mark.is_empty() {
            r.to_owned()
        } else {
            format!("{mark} {r}")
        };
        let mut available = area.right().saturating_sub(x + 1) as usize;
        if !hints.is_empty() {
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
            buf.set_string(area.right() - w - 1, area.y, &text, st);
            if !mark.is_empty() {
                buf.set_string(area.right() - w - 1, area.y, mark, mark_style);
            }

            let mut px = area.right() - w - 1;
            if !mark.is_empty() {
                px += crate::ui::text::width(mark) as u16 + 1;
            }
            let end = area.right() - 1;
            for (part, tone) in parts {
                let pw = crate::ui::text::width(part) as u16;
                let to = px.saturating_add(pw).min(end);
                if to > px {
                    buf.set_style(
                        Rect::new(px, area.y, to - px, 1),
                        ratatui::style::Style::new().fg(t.tone(*tone)),
                    );
                }
                px = px.saturating_add(pw);
            }
            right_w = w + 3;
        }
    }
    let limit = area.right().saturating_sub(right_w);
    let (keep, compact) = fit(hints, limit.saturating_sub(x), style);
    let cut = keep.len() < hints.len();
    if style.align == HintAlign::Center {
        let mut used = cost(hints, &keep, &compact) as u16;
        used = if cut {
            used.saturating_sub(GAP) + 1
        } else {
            used.saturating_sub(GAP)
        };
        let free = area.width.saturating_sub(used);
        let mid = area.x + free / 2;

        x = mid.max(x).min(limit.saturating_sub(used).max(x));
    }
    for &i in &keep {
        let h = &hints[i];
        let key = normalize_key(h.key);
        let key = t.glyphs.key_label(&key);
        let kw = crate::ui::text::width(&key) as u16;
        buf.set_string(x, area.y, &key, t.key_hint_key());
        if !compact[i] {
            buf.set_string(x + kw + 1, area.y, &h.action, t.key_hint_action());
        }
        let w = hint_width(h, compact[i]).min(area.right().saturating_sub(x));
        spots.push((i, Rect::new(x, area.y, w, 1)));
        x += hint_width(h, compact[i]) + GAP;
    }
    if cut && x < limit {
        buf.set_string(x, area.y, t.glyphs.ellipsis, t.faint());
    }
    keep.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hint_label_names_one_key_or_none() {
        use crate::core::event::Key;
        use ratatui::crossterm::event::KeyCode;
        let k = |code, mods| Some(Key { code, mods });
        assert_eq!(key_of("Esc"), k(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(
            key_of("Ctrl+S"),
            k(KeyCode::Char('s'), KeyModifiers::CONTROL)
        );
        assert_eq!(key_of("Shift+Tab"), k(KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(key_of("F1"), k(KeyCode::F(1), KeyModifiers::NONE));
        assert_eq!(key_of("?"), k(KeyCode::Char('?'), KeyModifiers::NONE));
        assert_eq!(key_of("G"), k(KeyCode::Char('G'), KeyModifiers::SHIFT));
        assert_eq!(key_of("Alt+↓"), k(KeyCode::Down, KeyModifiers::ALT));
        assert_eq!(key_of("Del"), k(KeyCode::Delete, KeyModifiers::NONE));
        for many in ["↑↓", "1–6", "Ctrl+B c", "Tab/Shift+Tab", "", "F13"] {
            assert_eq!(key_of(many), None, "{many}");
        }
    }

    #[test]
    fn width_shows_the_whole_bar() {
        let t = Theme::junie();
        let hints = [hint("Enter", "Open"), hint("Esc", "Back").pinned()];
        let w = width(&hints, Some("EDIT"), Some("! Connection lost"));
        let draw = |w: u16| {
            let area = Rect::new(0, 0, w, 1);
            let mut buf = Buffer::empty(area);
            render_toned(
                area,
                &mut buf,
                &t,
                &hints,
                Some(("EDIT", BadgeKind::Edit)),
                Some(("Connection lost", Tone::Error)),
            );
            (0..w).map(|x| buf[(x, 0)].symbol()).collect::<String>()
        };
        let row = draw(w);
        for part in ["EDIT", "Enter Open", "Esc Back", "! Connection lost"] {
            assert!(row.contains(part), "{row:?} lacks {part}");
        }
        let row = draw(w - 1);
        assert!(
            !["EDIT", "Enter Open", "Esc Back", "Connection lost"]
                .iter()
                .all(|p| row.contains(p)),
            "{row:?}"
        );
    }

    #[test]
    fn modifier_click_names_the_platform_modifier() {
        use crate::widgets::table::DataTable;
        let mac = ClientPlatform::MacOs;
        let other = ClientPlatform::Other;
        assert_eq!(
            modifier_click(DataTable::toggle_mods_for(mac), mac),
            Some("Option+click")
        );
        assert_eq!(
            modifier_click(DataTable::toggle_mods_for(other), other),
            Some("Ctrl+click")
        );

        assert_eq!(
            modifier_click(DataTable::TOGGLE_WITH_ALT, mac),
            Some("Option+click")
        );
        assert_eq!(
            modifier_click(DataTable::TOGGLE_WITH_ALT, other),
            Some("Ctrl+click")
        );
        assert_eq!(modifier_click(KeyModifiers::ALT, other), Some("Alt+click"));
        assert_eq!(modifier_click(KeyModifiers::SUPER, mac), Some("Cmd+click"));
        assert_eq!(modifier_click(KeyModifiers::SHIFT, mac), None);
    }

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
    fn a_status_made_of_parts_keeps_each_part_in_its_tone() {
        let t = Theme::junie();
        let parts = vec![
            ("deploy · ".to_owned(), Tone::Secondary),
            ("2 ok".to_owned(), Tone::Success),
            (" · ".to_owned(), Tone::Secondary),
            ("1 failed".to_owned(), Tone::Error),
        ];
        let text: String = parts.iter().map(|(p, _)| p.as_str()).collect();
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 1));
        render_parts(
            Rect::new(0, 0, 60, 1),
            &mut buf,
            &t,
            &[hint("Esc", "Back")],
            None,
            Some((&text, Tone::Error)),
            &parts,
            &t.hintbar,
        );
        let row = row_text(&buf, 60);
        assert!(row.contains("! deploy · 2 ok · 1 failed"), "{row:?}");
        let at = |needle: &str| row[..row.find(needle).unwrap()].chars().count() as u16;
        assert_eq!(buf[(at("!"), 0)].fg, t.error);
        assert_eq!(buf[(at("deploy"), 0)].fg, t.text_secondary);
        assert_eq!(buf[(at("2 ok"), 0)].fg, t.success);
        assert_eq!(buf[(at("1 failed") + 7, 0)].fg, t.error);
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
    fn a_status_kind_draws_its_glyph_and_label_from_theme_status() {
        let t = Theme::junie();
        for kind in StatusKind::ALL {
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
            render_status(
                *buf.area(),
                &mut buf,
                &t,
                &[],
                None,
                Some(("Saved", Some(kind))),
            );
            let (glyph, st) = t.status(kind);
            let row = row_text(&buf, 40);
            assert!(row.contains(&format!("{glyph} Saved")), "{kind:?}: {row:?}");
            let x = (0..40).find(|&x| buf[(x, 0)].symbol() == glyph).unwrap();
            assert_eq!(buf[(x, 0)].fg, st.fg.unwrap(), "{kind:?}");
            assert_eq!(
                buf[(x + 2, 0)].fg,
                t.status_label(kind).fg.unwrap(),
                "{kind:?}"
            );
        }

        let mut a = Buffer::empty(Rect::new(0, 0, 40, 1));
        let mut b = Buffer::empty(Rect::new(0, 0, 40, 1));
        render_status(*a.area(), &mut a, &t, &[], None, Some(("Copied", None)));
        render(*b.area(), &mut b, &t, &[], None, Some("Copied"));
        assert_eq!(a, b);
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

        assert!(row.find("Help") < row.find("Cancel"), "{row:?}");

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

        let mut narrow = t;
        narrow.hintbar.status_min_share = 90;
        let row = draw(72, &narrow, &hints, Some((status, Tone::Secondary)));
        assert!(!row.contains("Esc Back"), "{row:?}");

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

        let row = draw(40, &t, &[hint("Esc", "Back")], None);
        assert_eq!(row.find("Esc"), Some(16), "{row:?}");
        assert_eq!(row.len() - row.trim_end().len(), 16, "{row:?}");
    }
}
