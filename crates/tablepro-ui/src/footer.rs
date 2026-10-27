use termrock::{PaintStyle, Role, StylePatch, Ui, text::width};

const ACTION_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x008a_8a8a));

const OVERFLOW_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x0073_7373));

const BADGE_FG_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x0019_191c));

const BADGE_BG_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x0048_e054));

#[derive(Clone, Copy, Debug)]
pub(crate) struct Hint {
    pub key: &'static str,
    pub action: &'static str,
}

impl Hint {
    pub(crate) const fn new(key: &'static str, action: &'static str) -> Self {
        Self { key, action }
    }

    fn content_width(self) -> u16 {
        width(self.key) + 1 + width(self.action)
    }

    fn is_pinned(self) -> bool {
        matches!(self.key, "Esc" | "?" | "F1")
    }
}

const GAP: u16 = 2;

#[derive(Clone, Debug)]
pub(crate) struct Status {
    pub text: String,
    pub role: Role,
}

impl Status {
    pub(crate) fn new(text: impl Into<String>, role: Role) -> Self {
        Self {
            text: text.into(),
            role,
        }
    }
}

const OVERFLOW: &str = "…";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    pub groups: Vec<usize>,

    pub overflow_at: Option<u16>,
}

pub(crate) fn fit_hint_mask(costs: &[u16], room: u16, pinned: &[bool]) -> (Vec<bool>, bool) {
    let mut keep = vec![true; costs.len()];
    let cost = |keep: &[bool]| -> u16 {
        keep.iter()
            .zip(costs)
            .filter(|(on, _)| **on)
            .fold(0u16, |sum, (_, item)| sum.saturating_add(*item))
    };
    if cost(&keep) <= room {
        return (keep, false);
    }
    let mut order: Vec<usize> = (0..costs.len())
        .rev()
        .filter(|index| pinned.get(*index).copied() != Some(true))
        .collect();
    order.extend(
        (0..costs.len())
            .rev()
            .filter(|index| pinned.get(*index).copied() == Some(true)),
    );
    for index in order {
        let overflow = if keep.iter().any(|on| !on) { GAP } else { 0 };
        if cost(&keep).saturating_add(overflow) <= room {
            break;
        }
        if let Some(slot) = keep.get_mut(index) {
            *slot = false;
        }
    }
    let ellipsis = keep.iter().any(|on| !on);
    (keep, ellipsis)
}

pub(crate) fn plan(
    groups: &[Hint],
    area: termrock::Rect,
    status_width: u16,
    badge_advance: u16,
) -> Plan {
    if groups.is_empty() {
        return Plan {
            groups: Vec::new(),
            overflow_at: None,
        };
    }

    let limit = if status_width == 0 {
        area.right()
    } else {
        area.right().saturating_sub(status_width.saturating_add(3))
    };
    let start = area.x.saturating_add(1).saturating_add(badge_advance);
    let room = limit.saturating_sub(start);
    let costs: Vec<u16> = groups
        .iter()
        .map(|group| group.content_width().saturating_add(GAP))
        .collect();
    let pinned: Vec<bool> = groups.iter().map(|group| group.is_pinned()).collect();
    let (keep, _) = fit_hint_mask(&costs, room, &pinned);
    let kept: Vec<usize> = keep
        .iter()
        .enumerate()
        .filter(|(_, on)| **on)
        .map(|(index, _)| index)
        .collect();

    let mut x = start;
    for index in &kept {
        x = x.saturating_add(costs.get(*index).copied().unwrap_or(0));
    }
    let overflow_at = (kept.len() < groups.len() && x < limit).then_some(x);
    Plan {
        groups: kept,
        overflow_at,
    }
}

pub(crate) fn draw(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    groups: &[Hint],
    badge: Option<&str>,
    status: Option<&Status>,
    base: PaintStyle,
) {
    let status_width = status.map_or(0, |status| width(&status.text) as u16);

    let badge_advance = badge.map_or(0, |badge| width(badge).saturating_add(4));
    let plan = plan(groups, area, status_width, badge_advance);
    let action_style = base.patch(ui.paint_patch(&StylePatch::new().set_fg(ACTION_ROLE)));
    let key_style = base.add_modifier(termrock::Modifier::BOLD);
    let overflow_style = base.patch(ui.paint_patch(&StylePatch::new().set_fg(OVERFLOW_ROLE)));
    let mut x = area.x.saturating_add(1);
    if let Some(badge) = badge {
        let badge_style = base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(BADGE_FG_ROLE)
                    .set_bg(BADGE_BG_ROLE)
                    .add(termrock::Modifier::BOLD),
            ),
        );
        ui.paint_str(
            termrock::Rect {
                x,
                width: width(badge).saturating_add(2),
                height: 1,
                ..area
            },
            &format!(" {badge} "),
            badge_style,
        );
        x = x.saturating_add(width(badge)).saturating_add(4);
    }
    for index in &plan.groups {
        let Some(group) = groups.get(*index) else {
            continue;
        };
        ui.paint_str(
            termrock::Rect {
                x,
                width: width(group.key),
                height: 1,
                ..area
            },
            group.key,
            key_style,
        );
        ui.paint_str(
            termrock::Rect {
                x: x.saturating_add(width(group.key)).saturating_add(1),
                width: width(group.action),
                height: 1,
                ..area
            },
            group.action,
            action_style,
        );
        x = x.saturating_add(group.content_width()).saturating_add(GAP);
    }
    if let Some(overflow_at) = plan.overflow_at {
        ui.paint_str(
            termrock::Rect {
                x: overflow_at,
                width: 1,
                height: 1,
                ..area
            },
            OVERFLOW,
            overflow_style,
        );
    }
    if let Some(status) = status {
        let text_width = width(&status.text) as u16;
        let right = termrock::Rect {
            x: area.right().saturating_sub(text_width).saturating_sub(1),
            width: text_width,
            ..area
        };
        ui.paint_str(
            right,
            &status.text,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(status.role))),
        );
    }
}
