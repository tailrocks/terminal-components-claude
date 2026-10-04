//! Keyed Completion component for editor suggestion popups.
//!
//! Provides an anchored, non-modal completion suggestion list that tracks editor
//! anchor coordinates, flips when space below is insufficient, and emits typed apply/dismiss actions.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};

use crate::termrock::identity::{Id, ItemKey, Revision};
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::theme::StylePatch;

/// A single suggestion candidate in a [`Completion`] popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem<'a> {
    pub key: ItemKey,
    pub insert_text: &'a str,
    pub display_label: Option<&'a str>,
    pub detail: Option<&'a str>,
    pub icon: Option<&'a str>,
}

impl<'a> CompletionItem<'a> {
    pub fn new(key: impl Into<ItemKey>, insert_text: &'a str) -> Self {
        Self {
            key: key.into(),
            insert_text,
            display_label: None,
            detail: None,
            icon: None,
        }
    }

    pub fn display_label(mut self, label: &'a str) -> Self {
        self.display_label = Some(label);
        self
    }

    pub fn detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn label(&self) -> &str {
        self.display_label.unwrap_or(self.insert_text)
    }
}

/// Durable view state for [`Completion`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompletionState {
    pub highlighted: Option<ItemKey>,
    pub scroll: ScrollState,
    pub anchor: Position,
}

impl CompletionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_anchor(mut self, anchor: Position) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn highlighted(&self) -> Option<ItemKey> {
        self.highlighted
    }

    pub fn set_highlighted(&mut self, key: Option<ItemKey>) {
        self.highlighted = key;
    }

    pub fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    pub fn scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll
    }
}

/// Typed action emitted by [`Completion`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionAction {
    Apply { key: ItemKey, insert_text: String },
    Dismissed,
}

/// Anchored completion suggestion popup.
#[derive(Clone)]
pub struct Completion<'a> {
    pub id: Id,
    pub owner: Option<Id>,
    pub items: &'a [CompletionItem<'a>],
    pub revision: Revision,
    pub anchor: Option<Position>,
    pub patch: StylePatch,
    pub max_rows: u16,
}

impl<'a> Completion<'a> {
    pub fn new(id: Id, items: &'a [CompletionItem<'a>], revision: Revision) -> Self {
        Self {
            id,
            owner: None,
            items,
            revision,
            anchor: None,
            patch: StylePatch::empty(),
            max_rows: 8,
        }
    }

    pub fn owner(mut self, owner: Id) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn anchor(mut self, anchor: Position) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn max_rows(mut self, max: u16) -> Self {
        self.max_rows = max.max(1);
        self
    }

    pub fn row_id(&self, key: ItemKey) -> Id {
        self.id.sub("row").sub(&key.to_string())
    }

    /// Handles keyboard and pointer events for popup navigation and apply/dismiss.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletionState,
    ) -> Response<CompletionAction> {
        let keys: Vec<ItemKey> = self.items.iter().map(|item| item.key).collect();

        if let Some(h) = state.highlighted {
            if !keys.contains(&h) {
                state.highlighted = keys.first().copied();
            }
        } else {
            state.highlighted = keys.first().copied();
        }

        state.scroll.total = self.items.len();
        state.scroll.viewport = self.max_rows as usize;
        state.scroll.clamp();

        if let Some(h) = state.highlighted
            && let Some(pos) = self.items.iter().position(|item| item.key == h)
        {
            state.scroll.ensure_visible(pos);
        }

        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::Esc => {
                    cx.request_invalidate(Invalidate::Paint);
                    Response::action(self.id.clone(), CompletionAction::Dismissed)
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint)
                }
                KeyCode::Enter | KeyCode::Tab => {
                    if let Some(h) = state.highlighted
                        && let Some(item) = self.items.iter().find(|i| i.key == h)
                    {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(
                            self.id.clone(),
                            CompletionAction::Apply {
                                key: item.key,
                                insert_text: item.insert_text.to_string(),
                            },
                        )
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint);
                    }
                    Response::consumed(self.id.clone())
                }
                KeyCode::Up | KeyCode::Char('p')
                    if k.code != KeyCode::Char('p') || k.mods.contains(KeyModifiers::CONTROL) =>
                {
                    if !keys.is_empty() {
                        let curr_idx = state
                            .highlighted
                            .and_then(|curr| keys.iter().position(|&key| key == curr))
                            .unwrap_or(0);
                        let prev_idx = curr_idx.saturating_sub(1);
                        state.highlighted = Some(keys[prev_idx]);
                        state.scroll.ensure_visible(prev_idx);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::Down | KeyCode::Char('n')
                    if k.code != KeyCode::Char('n') || k.mods.contains(KeyModifiers::CONTROL) =>
                {
                    if !keys.is_empty() {
                        let curr_idx = state
                            .highlighted
                            .and_then(|curr| keys.iter().position(|&key| key == curr))
                            .unwrap_or(0);
                        let next_idx = (curr_idx + 1).min(keys.len().saturating_sub(1));
                        state.highlighted = Some(keys[next_idx]);
                        state.scroll.ensure_visible(next_idx);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::PageUp => {
                    if !keys.is_empty() {
                        let curr_idx = state
                            .highlighted
                            .and_then(|curr| keys.iter().position(|&key| key == curr))
                            .unwrap_or(0);
                        let new_idx = curr_idx.saturating_sub(self.max_rows as usize);
                        state.highlighted = Some(keys[new_idx]);
                        state.scroll.ensure_visible(new_idx);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::PageDown => {
                    if !keys.is_empty() {
                        let curr_idx = state
                            .highlighted
                            .and_then(|curr| keys.iter().position(|&key| key == curr))
                            .unwrap_or(0);
                        let new_idx =
                            (curr_idx + self.max_rows as usize).min(keys.len().saturating_sub(1));
                        state.highlighted = Some(keys[new_idx]);
                        state.scroll.ensure_visible(new_idx);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::Home => {
                    if let Some(&first) = keys.first() {
                        state.highlighted = Some(first);
                        state.scroll.ensure_visible(0);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::End => {
                    if let Some(&last) = keys.last() {
                        state.highlighted = Some(last);
                        state.scroll.ensure_visible(keys.len().saturating_sub(1));
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                _ => Response::bubble(self.id.clone()),
            },
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::from(m.pos);
                match m.kind {
                    crate::core::event::MouseKind::Down => {
                        for item in self.items {
                            if cx.contains_point(&self.row_id(item.key), pos) {
                                state.highlighted = Some(item.key);
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(
                                    self.id.clone(),
                                    CompletionAction::Apply {
                                        key: item.key,
                                        insert_text: item.insert_text.to_string(),
                                    },
                                )
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint);
                            }
                        }
                        if let Some(geom) = cx.published_geometry
                            && let Some(popup_rect) = geom.get(&self.id)
                            && !popup_rect.contains(pos)
                        {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), CompletionAction::Dismissed)
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint);
                        }
                        Response::consumed(self.id.clone())
                    }
                    crate::core::event::MouseKind::WheelUp => {
                        state.scroll.scroll_by(-1);
                        cx.request_invalidate(Invalidate::Paint);
                        Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                    }
                    crate::core::event::MouseKind::WheelDown => {
                        state.scroll.scroll_by(1);
                        cx.request_invalidate(Invalidate::Paint);
                        Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                    }
                    _ => Response::bubble(self.id.clone()),
                }
            }
            _ => Response::bubble(self.id.clone()),
        }
    }

    /// Measures the preferred size under constraints.
    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let max_content_w = self
            .items
            .iter()
            .map(|i| {
                let detail_len = i.detail.map(|d| d.chars().count() + 2).unwrap_or(0);
                i.label().chars().count() + detail_len + 6
            })
            .max()
            .unwrap_or(24) as u16;
        let w = max_content_w.clamp(24, 48).min(constraints.max.width);
        let h = (self.items.len().min(self.max_rows as usize) as u16).min(constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    /// Renders the anchored completion popup.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CompletionState) -> Rect {
        if self.items.is_empty() || area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let max_content_w = self
            .items
            .iter()
            .map(|i| {
                let detail_len = i.detail.map(|d| d.chars().count() + 2).unwrap_or(0);
                i.label().chars().count() + detail_len + 6
            })
            .max()
            .unwrap_or(24) as u16;
        let width = max_content_w.clamp(24, 48).min(area.width);
        let rows = (self.items.len().min(self.max_rows as usize) as u16).min(area.height);
        let height = rows.max(1);

        let anchor = self.anchor.unwrap_or(state.anchor);
        let mut x = anchor.x;
        let mut y = anchor.y.saturating_add(1);

        // Flip popup above anchor if space below is insufficient
        if y.saturating_add(height) > area.bottom() && anchor.y >= height {
            y = anchor.y.saturating_sub(height);
        }

        if x.saturating_add(width) > area.right() {
            x = area.right().saturating_sub(width);
        }
        if y.saturating_add(height) > area.bottom() {
            y = area.bottom().saturating_sub(height);
        }

        let popup_rect = Rect::new(x, y, width, height);
        ui.register_hit(self.id.clone(), popup_rect);

        let bg_style = Style::default()
            .bg(theme.tokens.surface_overlay)
            .fg(theme.tokens.text_primary);
        ui.fill_rect(popup_rect, bg_style);

        let visible_range = state.scroll.visible_range();
        for idx in visible_range {
            if idx >= self.items.len() {
                break;
            }
            let item = &self.items[idx];
            let row_y = y + (idx - state.scroll.offset) as u16;
            if row_y >= y + height {
                break;
            }

            let row_area = Rect::new(x, row_y, width, 1);
            ui.register_hit(self.row_id(item.key), row_area);

            let is_highlighted = state.highlighted == Some(item.key);
            let row_style = if is_highlighted {
                Style::default()
                    .bg(theme.tokens.highlight)
                    .fg(theme.tokens.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.tokens.text_primary)
            };

            ui.fill_rect(row_area, row_style);

            let icon = item.icon.unwrap_or("·");
            let lead = format!(" {icon} ");
            ui.set_string(x, row_y, &lead, row_style);

            let lead_w = lead.chars().count() as u16;
            ui.set_string(x + lead_w, row_y, item.label(), row_style);

            if let Some(detail) = item.detail {
                let detail_w = detail.chars().count() as u16;
                let detail_x = x + width.saturating_sub(detail_w + 1);
                if detail_x > x + lead_w + item.label().chars().count() as u16 {
                    let detail_style = if is_highlighted {
                        row_style
                    } else {
                        Style::default().fg(theme.tokens.text_muted)
                    };
                    ui.set_string(detail_x, row_y, detail, detail_style);
                }
            }
        }

        popup_rect
    }
}
