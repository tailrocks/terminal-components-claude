//! Keyed Picker component for search and item selection overlays.
//!
//! Provides query editing, keyboard and pointer navigation, item filtering,
//! readiness presentation (Loading, Error, Ready), and typed action emission.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};

use crate::termrock::empty::Readiness;
use crate::termrock::identity::{Id, ItemKey, Revision};
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::response::{ActivationOrigin, Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::theme::StylePatch;

/// A single candidate item in a [`Picker`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerItem<'a> {
    pub key: ItemKey,
    pub label: &'a str,
    pub detail: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub glyph: Option<&'a str>,
    pub group: Option<&'a str>,
    pub disabled: bool,
}

impl<'a> PickerItem<'a> {
    pub fn new(key: impl Into<ItemKey>, label: &'a str) -> Self {
        Self {
            key: key.into(),
            label,
            detail: None,
            tag: None,
            glyph: None,
            group: None,
            disabled: false,
        }
    }

    pub fn detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn tag(mut self, tag: &'a str) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn glyph(mut self, glyph: &'a str) -> Self {
        self.glyph = Some(glyph);
        self
    }

    pub fn group(mut self, group: &'a str) -> Self {
        self.group = Some(group);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Durable view state for [`Picker`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PickerState {
    pub query: String,
    pub highlighted: Option<ItemKey>,
    pub scroll: ScrollState,
}

impl PickerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    pub fn highlighted(&self) -> Option<ItemKey> {
        self.highlighted
    }

    pub fn set_highlighted(&mut self, key: Option<ItemKey>) {
        self.highlighted = key;
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    pub fn with_highlighted(mut self, key: impl Into<ItemKey>) -> Self {
        self.highlighted = Some(key.into());
        self
    }

    pub fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    pub fn scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll
    }
}

/// Typed result actions emitted by [`Picker`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerAction {
    Accept {
        key: ItemKey,
        alternate: bool,
        origin: ActivationOrigin,
    },
    ScopeNext,
    Back,
    Dismissed,
}

/// Searchable and fixed-choice overlay picker component.
#[derive(Clone)]
pub struct Picker<'a> {
    pub id: Id,
    pub items: &'a [PickerItem<'a>],
    pub revision: Revision,
    pub title: Option<&'a str>,
    pub query: Option<&'a str>,
    pub searchable: bool,
    pub readiness: Option<Readiness<'a>>,
    pub patch: StylePatch,
    pub max_rows: u16,
    pub width: Option<u16>,
    pub scope: Option<&'a str>,
    pub empty_text: Option<&'a str>,
}

impl<'a> Picker<'a> {
    pub fn new(id: Id, items: &'a [PickerItem<'a>], revision: Revision) -> Self {
        Self {
            id,
            items,
            revision,
            title: None,
            query: None,
            searchable: true,
            readiness: None,
            patch: StylePatch::empty(),
            max_rows: 8,
            width: None,
            scope: None,
            empty_text: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn readiness(mut self, readiness: Readiness<'a>) -> Self {
        self.readiness = Some(readiness);
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

    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn scope(mut self, scope: &'a str) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn empty_text(mut self, empty: &'a str) -> Self {
        self.empty_text = Some(empty);
        self
    }

    pub fn row_id(&self, key: ItemKey) -> Id {
        self.id.sub("row").sub(&key.to_string())
    }

    /// Filters items based on the active query.
    pub fn filtered_items(&self, query: &str) -> Vec<&'a PickerItem<'a>> {
        if !self.searchable || query.is_empty() {
            return self.items.iter().collect();
        }
        let q = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                item.label.to_lowercase().contains(&q)
                    || item
                        .detail
                        .map(|d| d.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || item
                        .group
                        .map(|g| g.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || item
                        .tag
                        .map(|t| t.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Handles events and updates durable view state.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut PickerState) -> Response<PickerAction> {
        if let Some(q) = self.query
            && state.query != q
        {
            state.query = q.to_string();
        }

        let filtered = self.filtered_items(&state.query);
        let eligible: Vec<ItemKey> = filtered
            .iter()
            .filter(|item| !item.disabled)
            .map(|item| item.key)
            .collect();

        // Reconcile highlighted item
        if let Some(h) = state.highlighted {
            if !eligible.contains(&h) {
                state.highlighted = eligible.first().copied();
            }
        } else {
            state.highlighted = eligible.first().copied();
        }

        state.scroll.total = filtered.len();
        state.scroll.viewport = self.max_rows as usize;
        state.scroll.clamp();

        if let Some(h) = state.highlighted
            && let Some(pos) = filtered.iter().position(|item| item.key == h)
        {
            state.scroll.ensure_visible(pos);
        }

        let is_status_only = matches!(
            self.readiness,
            Some(Readiness::Loading) | Some(Readiness::Error(_))
        );

        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::Esc => {
                    if self.searchable && !state.query.is_empty() {
                        state.query.clear();
                        let new_filtered = self.filtered_items(&state.query);
                        let new_eligible: Vec<ItemKey> = new_filtered
                            .iter()
                            .filter(|item| !item.disabled)
                            .map(|item| item.key)
                            .collect();
                        state.highlighted = new_eligible.first().copied();
                        state.scroll.total = new_filtered.len();
                        state.scroll.clamp();
                        cx.request_invalidate(Invalidate::Paint);
                        Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                    } else {
                        cx.request_invalidate(Invalidate::Paint);
                        Response::action(self.id.clone(), PickerAction::Dismissed)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint)
                    }
                }
                KeyCode::Enter => {
                    if is_status_only || filtered.is_empty() {
                        Response::consumed(self.id.clone())
                    } else if let Some(key) = state.highlighted {
                        let alt = k.mods.contains(KeyModifiers::ALT);
                        cx.request_invalidate(Invalidate::Paint);
                        Response::action(
                            self.id.clone(),
                            PickerAction::Accept {
                                key,
                                alternate: alt,
                                origin: ActivationOrigin::Keyboard,
                            },
                        )
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint)
                    } else {
                        Response::consumed(self.id.clone())
                    }
                }
                KeyCode::Tab => {
                    cx.request_invalidate(Invalidate::Paint);
                    Response::action(self.id.clone(), PickerAction::ScopeNext)
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint)
                }
                KeyCode::Up | KeyCode::Char('p')
                    if k.code != KeyCode::Char('p') || k.mods.contains(KeyModifiers::CONTROL) =>
                {
                    if !eligible.is_empty() {
                        if let Some(curr) = state.highlighted {
                            if let Some(idx) = eligible.iter().position(|&key| key == curr) {
                                let prev_idx = idx.saturating_sub(1);
                                state.highlighted = Some(eligible[prev_idx]);
                            } else {
                                state.highlighted = eligible.first().copied();
                            }
                        } else {
                            state.highlighted = eligible.first().copied();
                        }
                        if let Some(h) = state.highlighted
                            && let Some(pos) = filtered.iter().position(|item| item.key == h)
                        {
                            state.scroll.ensure_visible(pos);
                        }
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::Down | KeyCode::Char('n')
                    if k.code != KeyCode::Char('n') || k.mods.contains(KeyModifiers::CONTROL) =>
                {
                    if !eligible.is_empty() {
                        if let Some(curr) = state.highlighted {
                            if let Some(idx) = eligible.iter().position(|&key| key == curr) {
                                let next_idx = (idx + 1).min(eligible.len().saturating_sub(1));
                                state.highlighted = Some(eligible[next_idx]);
                            } else {
                                state.highlighted = eligible.first().copied();
                            }
                        } else {
                            state.highlighted = eligible.first().copied();
                        }
                        if let Some(h) = state.highlighted
                            && let Some(pos) = filtered.iter().position(|item| item.key == h)
                        {
                            state.scroll.ensure_visible(pos);
                        }
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::PageUp => {
                    if !eligible.is_empty() {
                        let curr_idx = state
                            .highlighted
                            .and_then(|curr| eligible.iter().position(|&key| key == curr))
                            .unwrap_or(0);
                        let new_idx = curr_idx.saturating_sub(self.max_rows as usize);
                        state.highlighted = Some(eligible[new_idx]);
                        if let Some(pos) = filtered
                            .iter()
                            .position(|item| item.key == eligible[new_idx])
                        {
                            state.scroll.ensure_visible(pos);
                        }
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::PageDown => {
                    if !eligible.is_empty() {
                        let curr_idx = state
                            .highlighted
                            .and_then(|curr| eligible.iter().position(|&key| key == curr))
                            .unwrap_or(0);
                        let new_idx = (curr_idx + self.max_rows as usize)
                            .min(eligible.len().saturating_sub(1));
                        state.highlighted = Some(eligible[new_idx]);
                        if let Some(pos) = filtered
                            .iter()
                            .position(|item| item.key == eligible[new_idx])
                        {
                            state.scroll.ensure_visible(pos);
                        }
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::Home => {
                    if let Some(&first) = eligible.first() {
                        state.highlighted = Some(first);
                        state.scroll.ensure_visible(0);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::End => {
                    if let Some(&last) = eligible.last() {
                        state.highlighted = Some(last);
                        if let Some(pos) = filtered.iter().position(|item| item.key == last) {
                            state.scroll.ensure_visible(pos);
                        }
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::Backspace => {
                    if self.searchable {
                        if !state.query.is_empty() {
                            state.query.pop();
                            let new_filtered = self.filtered_items(&state.query);
                            let new_eligible: Vec<ItemKey> = new_filtered
                                .iter()
                                .filter(|item| !item.disabled)
                                .map(|item| item.key)
                                .collect();
                            state.highlighted = new_eligible.first().copied();
                            state.scroll.total = new_filtered.len();
                            state.scroll.clamp();
                            cx.request_invalidate(Invalidate::Paint);
                            Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                        } else {
                            cx.request_invalidate(Invalidate::Paint);
                            Response::action(self.id.clone(), PickerAction::Back)
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint)
                        }
                    } else {
                        cx.request_invalidate(Invalidate::Paint);
                        Response::action(self.id.clone(), PickerAction::Back)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint)
                    }
                }
                KeyCode::Char(c)
                    if self.searchable
                        && !k.mods.contains(KeyModifiers::CONTROL)
                        && !k.mods.contains(KeyModifiers::ALT) =>
                {
                    state.query.push(c);
                    let new_filtered = self.filtered_items(&state.query);
                    let new_eligible: Vec<ItemKey> = new_filtered
                        .iter()
                        .filter(|item| !item.disabled)
                        .map(|item| item.key)
                        .collect();
                    state.highlighted = new_eligible.first().copied();
                    state.scroll.total = new_filtered.len();
                    state.scroll.clamp();
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                _ => Response::bubble(self.id.clone()),
            },
            UpdateCause::Input(Input::Paste(text), _) => {
                if self.searchable {
                    let sanitized: String = text
                        .chars()
                        .map(|c| {
                            if c == '\t' || c == '\n' || c == '\r' {
                                ' '
                            } else {
                                c
                            }
                        })
                        .collect();
                    state.query.push_str(&sanitized);
                    let new_filtered = self.filtered_items(&state.query);
                    let new_eligible: Vec<ItemKey> = new_filtered
                        .iter()
                        .filter(|item| !item.disabled)
                        .map(|item| item.key)
                        .collect();
                    state.highlighted = new_eligible.first().copied();
                    state.scroll.total = new_filtered.len();
                    state.scroll.clamp();
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                } else {
                    Response::bubble(self.id.clone())
                }
            }
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::from(m.pos);
                match m.kind {
                    crate::core::event::MouseKind::Down => {
                        // Check row hits
                        for item in &filtered {
                            if cx.contains_point(&self.row_id(item.key), pos) {
                                if !item.disabled && !is_status_only {
                                    state.highlighted = Some(item.key);
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        PickerAction::Accept {
                                            key: item.key,
                                            alternate: false,
                                            origin: ActivationOrigin::Pointer,
                                        },
                                    )
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                                }
                                return Response::consumed(self.id.clone());
                            }
                        }
                        // Check if click was outside the picker modal
                        if let Some(geom) = cx.published_geometry
                            && let Some(modal_rect) = geom.get(&self.id)
                            && !modal_rect.contains(pos)
                        {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), PickerAction::Dismissed)
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

    /// Measures the preferred dimensions under constraints.
    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let w = self.width.unwrap_or(60).min(constraints.max.width);
        let header_rows = if self.searchable { 3u16 } else { 1u16 };
        let h = (header_rows + self.max_rows + 1).min(constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }

    /// Renders the picker modal and registers hit regions.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &PickerState) -> Rect {
        if area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let filtered = self.filtered_items(&state.query);
        let header_rows = if self.searchable { 3u16 } else { 1u16 };
        let content_rows = (self.max_rows as usize).min(filtered.len().max(1)) as u16;
        let footer_rows = 1u16;

        let modal_w = self
            .width
            .unwrap_or(60)
            .min(area.width.saturating_sub(2))
            .max(10);
        let modal_h = (header_rows + content_rows + footer_rows).min(area.height);

        let modal_x = area.x + (area.width.saturating_sub(modal_w)) / 2;
        let modal_y = area.y + (area.height.saturating_sub(modal_h)) / 3;
        let modal_area = Rect::new(modal_x, modal_y, modal_w, modal_h);

        ui.register_hit(self.id.clone(), modal_area);
        ui.register_focus(self.id.clone(), true);

        // Fill modal surface background
        let bg_style = Style::default()
            .bg(theme.tokens.surface_overlay)
            .fg(theme.tokens.text_primary);
        ui.fill_rect(modal_area, bg_style);

        // Render Title Bar
        let title_style = Style::default()
            .fg(theme.tokens.accent)
            .add_modifier(Modifier::BOLD);
        let title_text = self.title.unwrap_or("Search");
        ui.set_string(modal_x + 2, modal_y, title_text, title_style);

        if let Some(scope) = self.scope {
            let scope_w = scope.chars().count() as u16;
            let scope_x = modal_x + modal_w.saturating_sub(scope_w + 2);
            let scope_style = Style::default().fg(theme.tokens.text_muted);
            ui.set_string(scope_x, modal_y, scope, scope_style);
        }

        let mut curr_y = modal_y + 1;

        // Render Search Query field if searchable
        if self.searchable {
            let prompt_style = Style::default()
                .fg(theme.tokens.accent)
                .add_modifier(Modifier::BOLD);
            ui.set_string(modal_x + 2, curr_y, "> ", prompt_style);

            let query_style = Style::default().fg(theme.tokens.text_primary);
            ui.set_string(modal_x + 4, curr_y, &state.query, query_style);

            let cursor_pos = Position::new(
                (modal_x + 4 + state.query.chars().count() as u16).min(modal_x + modal_w - 2),
                curr_y,
            );
            ui.request_cursor(self.id.clone(), cursor_pos);

            curr_y += 1;

            // Separator line
            let sep_style = Style::default().fg(theme.tokens.border_subtle);
            let sep_line: String = "─".repeat(modal_w.saturating_sub(2) as usize);
            ui.set_string(modal_x + 1, curr_y, &sep_line, sep_style);

            curr_y += 1;
        }

        // Render items or readiness state
        if let Some(Readiness::Loading) = &self.readiness {
            let load_style = Style::default().fg(theme.tokens.text_muted);
            ui.set_string(modal_x + 2, curr_y, "⠋ Loading...", load_style);
        } else if let Some(Readiness::Error(msg)) = &self.readiness {
            let err_style = Style::default().fg(theme.tokens.danger);
            ui.set_string(modal_x + 2, curr_y, &format!("! {msg}"), err_style);
        } else if filtered.is_empty() {
            let empty = self.empty_text.unwrap_or("No matching results");
            let empty_style = Style::default().fg(theme.tokens.text_muted);
            ui.set_string(modal_x + 2, curr_y, empty, empty_style);
        } else {
            let visible_range = state.scroll.visible_range();

            for (row_y, idx) in (curr_y..).zip(visible_range) {
                if idx >= filtered.len() || row_y >= modal_y + modal_h.saturating_sub(footer_rows) {
                    break;
                }
                let item = filtered[idx];
                let is_highlighted = state.highlighted == Some(item.key);
                let row_area = Rect::new(modal_x + 1, row_y, modal_w.saturating_sub(2), 1);
                ui.register_hit(self.row_id(item.key), row_area);

                let row_style = if is_highlighted {
                    Style::default()
                        .bg(theme.tokens.highlight)
                        .fg(theme.tokens.accent)
                        .add_modifier(Modifier::BOLD)
                } else if item.disabled {
                    Style::default().fg(theme.tokens.disabled)
                } else {
                    Style::default().fg(theme.tokens.text_primary)
                };

                ui.fill_rect(row_area, row_style);

                let marker = if is_highlighted { "›" } else { " " };
                let glyph = item.glyph.unwrap_or("·");
                let lead = format!("{marker} {glyph} ");
                ui.set_string(modal_x + 2, row_y, &lead, row_style);

                let lead_w = lead.chars().count() as u16;
                ui.set_string(modal_x + 2 + lead_w, row_y, item.label, row_style);

                let mut trail_offset = modal_w.saturating_sub(3);

                if let Some(tag) = item.tag {
                    let tag_w = tag.chars().count() as u16;
                    let tag_x = modal_x + trail_offset.saturating_sub(tag_w);
                    let tag_style = if is_highlighted {
                        row_style
                    } else {
                        Style::default().fg(theme.tokens.accent)
                    };
                    ui.set_string(tag_x, row_y, tag, tag_style);
                    trail_offset = trail_offset.saturating_sub(tag_w + 1);
                }

                if let Some(detail) = item.detail {
                    let detail_w = detail.chars().count() as u16;
                    let detail_x = modal_x + trail_offset.saturating_sub(detail_w);
                    let detail_style = if is_highlighted {
                        row_style
                    } else {
                        Style::default().fg(theme.tokens.text_muted)
                    };
                    if detail_x > modal_x + 2 + lead_w + item.label.chars().count() as u16 {
                        ui.set_string(detail_x, row_y, detail, detail_style);
                    }
                }
            }
        }

        // Render footer hint row
        let footer_y = modal_y + modal_h.saturating_sub(1);
        let footer_style = Style::default().fg(theme.tokens.text_muted);
        let footer_text = "↑↓ Navigate  ↵ Select  Alt+↵ Alt  Tab Scope  Esc Dismiss";
        ui.set_string(modal_x + 2, footer_y, footer_text, footer_style);

        modal_area
    }
}
