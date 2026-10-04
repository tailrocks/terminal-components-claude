//! Termrock chrome-only container panel component.
//!
//! Provides the canonical [`Panel`], [`PanelKind`], and [`Badge`] supporting
//! Card and Framed recipes with title, metadata, badge, focus-within chrome,
//! and clipped body slot.

use crate::termrock::author::StyledText;
use crate::termrock::identity::{Id, Part};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::runtime::{MeasureCx, Ui};
use crate::termrock::text::width;
use crate::termrock::theme::{Role, StylePatch, Surface, Tone};

/// Visual flavor of a panel container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelKind {
    /// Filled surface rectangle with padding and no outer border. Default.
    #[default]
    Card,
    /// Rounded subtle border surface used for distinct panes and splits.
    Framed,
}

/// Metadata badge displayed in the title row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Badge<'a> {
    pub label: &'a str,
    pub tone: Option<Tone>,
}

impl<'a> Badge<'a> {
    pub const fn new(label: &'a str) -> Self {
        Self { label, tone: None }
    }

    pub const fn with_tone(label: &'a str, tone: Tone) -> Self {
        Self {
            label,
            tone: Some(tone),
        }
    }
}

impl<'a> From<&'a str> for Badge<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

/// Cells a title keeps when competing with long metadata for the title row.
const TITLE_MIN: usize = 4;

/// Display-only container panel providing Card or Framed chrome around a child body.
#[derive(Debug, Clone)]
pub struct Panel<'a> {
    pub id: Id,
    pub kind: PanelKind,
    pub title: Option<&'a str>,
    pub meta: Option<StyledText<'a>>,
    pub badge: Option<Badge<'a>>,
    pub focus_within: bool,
    pub patch: Option<StylePatch>,
}

impl<'a> Panel<'a> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            kind: PanelKind::Card,
            title: None,
            meta: None,
            badge: None,
            focus_within: false,
            patch: None,
        }
    }

    pub fn card(id: Id) -> Self {
        Self::new(id).kind(PanelKind::Card)
    }

    pub fn framed(id: Id) -> Self {
        Self::new(id).kind(PanelKind::Framed)
    }

    pub fn kind(mut self, kind: PanelKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn meta(mut self, meta: StyledText<'a>) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn badge(mut self, badge: Option<Badge<'a>>) -> Self {
        self.badge = badge;
        self
    }

    pub fn focus_within(mut self, focus_within: bool) -> Self {
        self.focus_within = focus_within;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Measure required panel size around child content.
    pub fn measure(&self, _cx: &MeasureCx<'_>, child: Size, constraints: Constraints) -> Size {
        let (extra_w, extra_h) = match self.kind {
            PanelKind::Card => {
                let h = if self.title.is_some() || self.meta.is_some() || self.badge.is_some() {
                    3
                } else {
                    2
                };
                (4, h)
            }
            PanelKind::Framed => (5, 2),
        };
        let needed = Size::new(
            child.width.saturating_add(extra_w),
            child.height.saturating_add(extra_h),
        );
        constraints.clamp(needed)
    }

    /// Draw panel chrome and invoke body callback within the clipped inner allocation.
    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> R {
        if area.is_empty() {
            return body(ui, Rect::zero());
        }

        let surface = match self.kind {
            PanelKind::Card => Surface::Elevated,
            PanelKind::Framed => Surface::Canvas,
        };

        ui.with_surface(surface, |ui| {
            let bg_color = match surface {
                Surface::Elevated => ui.theme.tokens.surface_elevated,
                Surface::Canvas => ui.theme.tokens.canvas,
                _ => ui.theme.tokens.surface,
            };
            let bg_style = ratatui::style::Style::new().bg(bg_color);
            ui.fill_rect(area, bg_style);

            let body_area = match self.kind {
                PanelKind::Card => {
                    let inner = area.inset(2, 1);
                    if self.focus_within
                        && (self.title.is_some() || self.meta.is_some() || self.badge.is_some())
                        && area.width > 2
                    {
                        let focus_style = ui.theme.resolve_style(Role::Accent, surface);
                        ui.set_string(area.x + 1, area.y, "▎", focus_style);
                    }
                    if area.width > 4 {
                        self.draw_title_row(
                            ui,
                            area.x + 2,
                            area.y,
                            area.width.saturating_sub(4),
                            surface,
                        );
                    }
                    if self.title.is_some() || self.meta.is_some() || self.badge.is_some() {
                        Rect::new(
                            inner.x,
                            inner.y.saturating_add(1),
                            inner.width,
                            inner.height.saturating_sub(1),
                        )
                    } else {
                        inner
                    }
                }
                PanelKind::Framed => {
                    if area.width >= 2 && area.height >= 2 {
                        let border_role = if self.focus_within {
                            Role::BorderFocused
                        } else {
                            Role::Border
                        };
                        let border_style = ui.theme.resolve_style(border_role, surface);

                        ui.set_string(area.x, area.y, "╭", border_style);
                        ui.set_string(area.right().saturating_sub(1), area.y, "╮", border_style);
                        ui.set_string(area.x, area.bottom().saturating_sub(1), "╰", border_style);
                        ui.set_string(
                            area.right().saturating_sub(1),
                            area.bottom().saturating_sub(1),
                            "╯",
                            border_style,
                        );

                        for x in (area.x + 1)..area.right().saturating_sub(1) {
                            ui.set_string(x, area.y, "─", border_style);
                            ui.set_string(x, area.bottom().saturating_sub(1), "─", border_style);
                        }
                        for y in (area.y + 1)..area.bottom().saturating_sub(1) {
                            ui.set_string(area.x, y, "│", border_style);
                            ui.set_string(area.right().saturating_sub(1), y, "│", border_style);
                        }
                    }

                    if area.width > 4 {
                        self.draw_title_row(
                            ui,
                            area.x + 2,
                            area.y,
                            area.width.saturating_sub(4),
                            surface,
                        );
                    }

                    let inner = area.inset(1, 1);
                    Rect::new(
                        inner.x.saturating_add(2),
                        inner.y,
                        inner.width.saturating_sub(3),
                        inner.height,
                    )
                }
            };

            ui.clip(body_area, |clipped_ui| body(clipped_ui, body_area))
        })
    }

    fn draw_title_row(&self, ui: &mut Ui<'_>, x: u16, y: u16, w: u16, surface: Surface) {
        if w == 0 {
            return;
        }

        let framed = self.kind == PanelKind::Framed;
        let pad: u16 = if framed { 2 } else { 0 };

        let title_min = self
            .title
            .map(|t| width(t).min(TITLE_MIN) as u16)
            .unwrap_or(0);

        let meta_str = self.meta.as_ref().map(|m| m.text);
        let truncated_meta = meta_str.map(|m| {
            let room = w.saturating_sub(pad + if title_min > 0 { title_min + 1 } else { 0 });
            if width(m) as u16 > room {
                crate::termrock::text::truncate(m, room as usize).into_owned()
            } else {
                m.to_owned()
            }
        });

        let meta_w = truncated_meta
            .as_ref()
            .map(|m| width(m) as u16 + pad)
            .unwrap_or(0);

        let mut cx = x;

        if let Some(title) = self.title {
            let title_role = if self.focus_within {
                Role::Text
            } else {
                Role::TextMuted
            };
            let mut style = ui.theme.resolve_style(title_role, surface);
            if self.focus_within {
                style = style.add_modifier(ratatui::style::Modifier::BOLD);
            }

            let room = if meta_w > 0 {
                w.saturating_sub(meta_w + 1 + pad)
            } else {
                w.saturating_sub(pad)
            };
            let truncated_title = crate::termrock::text::truncate(title, room as usize);
            let display_title = if framed {
                format!(" {truncated_title} ")
            } else {
                truncated_title.to_string()
            };

            ui.part(
                self.id.clone(),
                Part::new("title"),
                Rect::new(cx, y, width(&display_title) as u16, 1),
                |_p| {},
            );
            ui.set_string(cx, y, &display_title, style);
            cx = cx.saturating_add(width(&display_title) as u16);
        }

        let mut right = x.saturating_add(w);

        if let Some(ref meta) = truncated_meta {
            let display_meta = if framed {
                format!(" {meta} ")
            } else {
                meta.clone()
            };
            let mw = width(&display_meta) as u16;
            if right >= cx.saturating_add(mw).saturating_add(u16::from(cx > x)) {
                right = right.saturating_sub(mw);
                let meta_style = ui.theme.resolve_style(Role::TextMuted, surface);
                ui.part(
                    self.id.clone(),
                    Part::new("meta"),
                    Rect::new(right, y, mw, 1),
                    |_p| {},
                );
                ui.set_string(right, y, &display_meta, meta_style);
            }
        }

        if let Some(ref badge) = self.badge {
            let display_badge = format!(" {} ", badge.label);
            let bw = width(&display_badge) as u16;
            if right > cx.saturating_add(bw).saturating_add(1) {
                right = right.saturating_sub(bw + 1);
                let badge_role = match badge.tone {
                    Some(Tone::Accent) => Role::Accent,
                    Some(Tone::Success) => Role::Success,
                    Some(Tone::Warning) => Role::Warning,
                    Some(Tone::Danger) => Role::Danger,
                    _ => Role::Secondary,
                };
                let badge_style = ui.theme.resolve_style(badge_role, surface);
                ui.part(
                    self.id.clone(),
                    Part::BADGE,
                    Rect::new(right, y, bw, 1),
                    |_p| {},
                );
                ui.set_string(right, y, &display_badge, badge_style);
            }
        }
    }
}
