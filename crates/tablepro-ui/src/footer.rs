//! The workbench footer: the hint groups on the left, the status on the right.
//!
//! The footer follows the shared hint bar's fitting rule:
//!
//! * The status owns the right edge. Its width is reserved before the hints
//!   are placed, so a hint never runs under it.
//! * The hint groups fill what is left, from the left.
//! * When they do not fit, the groups that fall off the right end are
//!   dropped — the pinned ones (`Esc`, `?`, `F1`) go last — and an overflow
//!   marker takes the first of the freed columns.
//!
//! The app owns the *content* (which groups a surface offers, and what its
//! status says). The *composition* — how a group is measured, how the status
//! is reserved, and where the overflow marker goes — lives here so every
//! surface renders the same way.

use termrock::{PaintStyle, Role, StylePatch, Ui, text::width};

/// The ink a hint action is written in.
const ACTION_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x008a_8a8a));
/// The ink the overflow marker is written in.
const OVERFLOW_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x0073_7373));
/// The ink the edit badge is written in.
const BADGE_FG_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x0019_191c));
/// The ground the edit badge sits on.
const BADGE_BG_ROLE: Role = Role::Custom(termrock::Color::from_u32(0x0048_e054));

/// One hint group: a key, a space, and an action.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hint {
    pub key: &'static str,
    pub action: &'static str,
}

impl Hint {
    pub(crate) const fn new(key: &'static str, action: &'static str) -> Self {
        Self { key, action }
    }

    /// The width the group occupies, without the gap that follows it.
    fn content_width(self) -> u16 {
        width(self.key) + 1 + width(self.action)
    }

    fn is_pinned(self) -> bool {
        matches!(self.key, "Esc" | "?" | "F1")
    }
}

/// The gap between two hint groups, and between a group and the overflow
/// marker.
const GAP: u16 = 2;

/// The status a surface shows on the right of the footer.
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

/// The overflow marker that replaces the groups that did not fit.
const OVERFLOW: &str = "…";

/// How the footer divides the width it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    /// The hint groups that are shown, in order.
    pub groups: Vec<usize>,
    /// Where the overflow marker goes, when the groups did not all fit.
    pub overflow_at: Option<u16>,
}

/// Fit the hint groups beside a status of `status_width` columns.
///
/// The status always wins the right edge. The groups fill what is left, from
/// the left; the ones that fall off the right end are dropped, and the
/// overflow marker takes the first of those columns.
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
    // The status is separated from the last group by a gap.
    let limit = if status_width == 0 {
        area.right()
    } else {
        area.right().saturating_sub(status_width.saturating_add(3))
    };
    let start = area.x.saturating_add(1).saturating_add(badge_advance);
    let room = limit.saturating_sub(start);
    let width_of = |index: &usize| {
        groups
            .get(*index)
            .map_or(0, |group| group.content_width().saturating_add(GAP))
    };
    let cost = |keep: &[usize]| -> u16 {
        let mut total: u16 = keep.iter().map(width_of).sum();
        if keep.len() < groups.len() {
            total = total.saturating_add(GAP);
        }
        total
    };
    let mut keep: Vec<usize> = (0..groups.len()).collect();
    if cost(&keep) > room {
        // The groups that fall off the right end go first; the pinned ones
        // survive longest.
        let mut order: Vec<usize> = (0..groups.len())
            .rev()
            .filter(|index| !groups.get(*index).is_some_and(|group| group.is_pinned()))
            .collect();
        order.extend(
            (0..groups.len())
                .rev()
                .filter(|index| groups.get(*index).is_some_and(|group| group.is_pinned())),
        );
        for gone in order {
            if cost(&keep) <= room {
                break;
            }
            keep.retain(|index| *index != gone);
        }
    }
    // The overflow marker takes the first of the freed columns.
    let mut x = start;
    for index in &keep {
        x = x.saturating_add(width_of(index));
    }
    let overflow_at = (keep.len() < groups.len() && x < limit).then_some(x);
    Plan {
        groups: keep,
        overflow_at,
    }
}

/// Paint the footer: the hint groups on the left, the status on the right.
///
/// `badge` is the small plate that sits before the first group (`EDIT` while
/// a field owns the keyboard). `status` is the message on the right edge.
pub(crate) fn draw(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    groups: &[Hint],
    badge: Option<&str>,
    status: Option<&Status>,
    base: PaintStyle,
) {
    let status_width = status.map_or(0, |status| width(&status.text) as u16);
    // The badge sits in a cell of its own, then a gap before the first group.
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
