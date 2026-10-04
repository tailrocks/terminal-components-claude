//! Status Bar chrome component.
//!
//! Paints one full-width bottom status row divided into left, center, and right groups.
//! Implements priority-based responsive degradation and interactive hit handling.

use ratatui::style::{Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::{ActionKey, Id, ItemKey};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::meter::{MeterLevel, MeterTone};
use crate::termrock::response::{ActivationOrigin, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{AnimationSample, Cx, MeasureCx, Ui};
use crate::termrock::spinner::spinner_frame;
use crate::termrock::theme::{StylePatch, Tone};

const GAP: u16 = 3;
const EDGE: u16 = 1;
const STATUS_METER_TRACK: u16 = 8;

/// Item alignment within status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Visual emphasis of a status item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusEmphasis {
    #[default]
    Plain,
    Strong,
    Chip,
}

/// A single item rendered in the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItem<'a> {
    pub key: ItemKey,
    pub text: &'a str,
    pub alignment: StatusAlignment,
    pub priority: u8,
    pub id: Option<Id>,
    pub tone: Tone,
    pub emphasis: StatusEmphasis,
    pub busy: bool,
    pub meter: Option<(Option<u8>, MeterTone)>,
}

impl<'a> StatusItem<'a> {
    pub fn new(key: impl Into<ItemKey>, text: &'a str) -> Self {
        Self {
            key: key.into(),
            text,
            alignment: StatusAlignment::Left,
            priority: 5,
            id: None,
            tone: Tone::Normal,
            emphasis: StatusEmphasis::Plain,
            busy: false,
            meter: None,
        }
    }

    pub fn alignment(mut self, alignment: StatusAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn interactive(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn strong(mut self) -> Self {
        self.emphasis = StatusEmphasis::Strong;
        self
    }

    pub fn chip(mut self) -> Self {
        self.emphasis = StatusEmphasis::Chip;
        self
    }

    pub fn busy(mut self) -> Self {
        self.busy = true;
        self
    }

    pub fn meter(mut self, used_pct: Option<u8>, tone: MeterTone) -> Self {
        self.meter = Some((used_pct, tone));
        self
    }

    pub fn width(&self) -> u16 {
        let base_w = UnicodeWidthStr::width(self.text) as u16 + if self.busy { 2 } else { 0 };
        let w = match self.emphasis {
            StatusEmphasis::Chip => base_w + 2,
            _ => base_w,
        };
        if self.meter.is_some() {
            w + 1 + STATUS_METER_TRACK + 7
        } else {
            w
        }
    }
}

/// Action emitted when interacting with an interactive status item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusBarAction {
    ItemClicked(ItemKey),
    Invoke {
        key: ItemKey,
        action: ActionKey,
        origin: ActivationOrigin,
    },
}

impl StatusBarAction {
    pub fn item_key(&self) -> ItemKey {
        match self {
            Self::ItemClicked(k) => *k,
            Self::Invoke { key, .. } => *key,
        }
    }
}

/// Caller-owned state for the status bar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusBarState {
    pub scroll_offset: u16,
    pub highlighted: Option<ItemKey>,
}

impl StatusBarState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Placed item description for status bar layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedStatusItem<'a> {
    pub item: &'a StatusItem<'a>,
    pub x: u16,
    pub width: u16,
    pub truncated_text: String,
}

/// Status bar chrome component.
#[derive(Clone)]
pub struct StatusBar<'a> {
    pub id: Id,
    pub left: &'a [StatusItem<'a>],
    pub center: &'a [StatusItem<'a>],
    pub right: &'a [StatusItem<'a>],
    pub animation: Option<AnimationSample>,
    pub patch: StylePatch,
}

impl<'a> StatusBar<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            left: &[],
            center: &[],
            right: &[],
            animation: None,
            patch: StylePatch::empty(),
        }
    }

    pub fn left(mut self, left: &'a [StatusItem<'a>]) -> Self {
        self.left = left;
        self
    }

    pub fn center(mut self, center: &'a [StatusItem<'a>]) -> Self {
        self.center = center;
        self
    }

    pub fn right(mut self, right: &'a [StatusItem<'a>]) -> Self {
        self.right = right;
        self
    }

    pub fn animation(mut self, sample: AnimationSample) -> Self {
        self.animation = Some(sample);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn row_id(&self, key: ItemKey) -> Id {
        self.id.sub("item").sub(&key.to_string())
    }

    /// Solves item placement and responsive dropping order.
    pub fn layout(&self, area: Rect) -> Vec<PlacedStatusItem<'a>> {
        let total_w = area.width;
        let mut keep = [
            vec![true; self.left.len()],
            vec![true; self.center.len()],
            vec![true; self.right.len()],
        ];

        let groups = [self.left, self.center, self.right];
        let group_w = |g_idx: usize, keep_arr: &[Vec<bool>; 3]| -> u16 {
            let items = groups[g_idx];
            let mut w = 0u16;
            let mut n = 0u16;
            for (it, k) in items.iter().zip(&keep_arr[g_idx]) {
                if *k {
                    w += it.width();
                    n += 1;
                }
            }
            if n > 0 { w + (n - 1) * GAP } else { 0 }
        };

        let needed = |keep_arr: &[Vec<bool>; 3]| -> u16 {
            let ws = [
                group_w(0, keep_arr),
                group_w(1, keep_arr),
                group_w(2, keep_arr),
            ];
            let present = ws.iter().filter(|w| **w > 0).count() as u16;
            let gaps = present.saturating_sub(1) * GAP;
            ws.iter().sum::<u16>() + gaps + 2 * EDGE
        };

        // Drop lowest priority item first; ties leave center first, then right, then left;
        // The first left item never leaves (it truncates instead).
        while needed(&keep) > total_w {
            let mut victim: Option<(u8, usize, usize)> = None;
            for g in [1usize, 2, 0] {
                let items = groups[g];
                let alive: Vec<usize> = (0..items.len()).filter(|i| keep[g][*i]).collect();
                if alive.is_empty() || (g == 0 && alive.len() == 1) {
                    continue;
                }
                let i = alive
                    .iter()
                    .copied()
                    .min_by_key(|&idx| (items[idx].priority, usize::MAX - idx))
                    .unwrap();
                let p = items[i].priority;
                if victim.is_none_or(|(vp, _, _)| p < vp) {
                    victim = Some((p, g, i));
                }
            }
            match victim {
                Some((_, g, i)) => keep[g][i] = false,
                None => break,
            }
        }

        let mut placed = Vec::new();

        // 1. Left group
        let mut x = area.x + EDGE;
        let left_budget = total_w.saturating_sub(2 * EDGE);
        for (i, it) in self.left.iter().enumerate() {
            if !keep[0][i] {
                continue;
            }
            let mut w = it.width();
            let mut text = it.text.to_string();
            let room = (area.x + EDGE + left_budget).saturating_sub(x);
            if w > room {
                let pad = if it.emphasis == StatusEmphasis::Chip {
                    2
                } else {
                    0
                } + if it.busy { 2 } else { 0 };
                let available = room.saturating_sub(pad) as usize;
                text = it.text.chars().take(available).collect();
                w = UnicodeWidthStr::width(text.as_str()) as u16 + pad;
            }
            placed.push(PlacedStatusItem {
                item: it,
                x,
                width: w,
                truncated_text: text,
            });
            x += w + GAP;
        }
        let left_end = x.saturating_sub(GAP);

        // 2. Right group
        let mut rx = area.right().saturating_sub(EDGE);
        let mut right_items = Vec::new();
        for (i, it) in self.right.iter().enumerate().rev() {
            if !keep[2][i] {
                continue;
            }
            let w = it.width();
            rx = rx.saturating_sub(w);
            right_items.push(PlacedStatusItem {
                item: it,
                x: rx,
                width: w,
                truncated_text: it.text.to_string(),
            });
            rx = rx.saturating_sub(GAP);
        }
        let right_start = if right_items.is_empty() {
            area.right().saturating_sub(EDGE)
        } else {
            rx + GAP
        };

        // 3. Center group: centred in free span
        let cw = group_w(1, &keep);
        if cw > 0 {
            let lo = left_end + GAP;
            let hi = right_start.saturating_sub(GAP);
            let free = hi.saturating_sub(lo);
            let mut cx_x = lo + free.saturating_sub(cw) / 2;
            for (i, it) in self.center.iter().enumerate() {
                if !keep[1][i] {
                    continue;
                }
                let w = it.width();
                placed.push(PlacedStatusItem {
                    item: it,
                    x: cx_x,
                    width: w,
                    truncated_text: it.text.to_string(),
                });
                cx_x += w + GAP;
            }
        }

        // Add right items
        placed.extend(right_items.into_iter().rev());
        placed
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        _state: &mut StatusBarState,
    ) -> Response<StatusBarAction> {
        let resp = Response::new(self.id.clone());

        if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause
            && m.kind == crate::core::event::MouseKind::Down
        {
            let pos = crate::termrock::layout::Position::from(m.pos);
            if let Some(geom) = cx.published_geometry {
                // Check all items
                let all_items = self.left.iter().chain(self.center).chain(self.right);
                for it in all_items {
                    if it.id.is_some() {
                        let item_id = self.row_id(it.key);
                        if let Some(rect) = geom.get(&item_id)
                            && rect.contains(pos)
                        {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                StatusBarAction::ItemClicked(it.key),
                            )
                            .with_invalidate(Invalidate::Paint);
                        }
                    }
                }
            }
        }

        resp
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        constraints.clamp(Size::new(constraints.max.width, 1))
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }

        let theme = ui.theme;
        let bg_style = Style::default().bg(theme.tokens.surface);
        ui.fill_rect(area, bg_style);

        let placed = self.layout(area);
        let tick = self.animation.map(|a| a.index).unwrap_or(0);

        for p in placed {
            let it = p.item;
            let item_area = Rect::new(p.x, area.y, p.width, 1);
            if it.id.is_some() {
                ui.register_hit(self.row_id(it.key), item_area);
            }

            let mut curr_x = p.x;

            // Draw chip background if applicable
            if it.emphasis == StatusEmphasis::Chip {
                let chip_bg = Style::default().bg(theme.tokens.surface_overlay);
                ui.fill_rect(item_area, chip_bg);
                curr_x += 1;
            }

            // Draw spinner if busy
            if it.busy {
                let frame = spinner_frame(tick);
                let sp_style = Style::default().fg(theme.tokens.accent);
                ui.set_string(curr_x, area.y, frame, sp_style);
                curr_x += 2;
            }

            // Determine text color based on tone
            let text_color = match it.tone {
                Tone::Primary | Tone::Normal => theme.tokens.text_primary,
                Tone::Accent => theme.tokens.accent,
                Tone::Success => theme.tokens.success,
                Tone::Warning => theme.tokens.warning,
                Tone::Danger => theme.tokens.danger,
                Tone::Muted => theme.tokens.text_muted,
            };

            let mut text_style = Style::default().fg(text_color);
            if it.emphasis == StatusEmphasis::Strong {
                text_style = text_style.add_modifier(Modifier::BOLD);
            }

            ui.set_string(curr_x, area.y, &p.truncated_text, text_style);
            curr_x += UnicodeWidthStr::width(p.truncated_text.as_str()) as u16;

            // Draw inline meter if specified
            if let Some((used_pct, tone)) = it.meter {
                curr_x += 1;
                let pct = used_pct.unwrap_or(0);
                let ratio = (pct.min(100) as f32) / 100.0;
                let filled = ((STATUS_METER_TRACK as f32) * ratio).round() as u16;
                let l = tone.level(used_pct).unwrap_or(MeterLevel::Low);
                let meter_color = match l {
                    MeterLevel::Low => theme.tokens.text_secondary,
                    MeterLevel::Medium => theme.tokens.warning,
                    MeterLevel::High => theme.tokens.danger,
                };

                for i in 0..STATUS_METER_TRACK {
                    let sym = if i < filled { "━" } else { "─" };
                    let st = if i < filled {
                        Style::default().fg(meter_color)
                    } else {
                        Style::default().fg(theme.tokens.border_subtle)
                    };
                    ui.set_string(curr_x + i, area.y, sym, st);
                }
                curr_x += STATUS_METER_TRACK;

                let val_str = format!(" {pct:>3}%");
                ui.set_string(
                    curr_x,
                    area.y,
                    &val_str,
                    Style::default().fg(theme.tokens.text_secondary),
                );
            }
        }

        area
    }
}
