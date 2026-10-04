//! Keyed CommandPalette component for modal command search and execution.
//!
//! Reuses the shared [`PickerState`] mechanism while specializing presentation for
//! commands with keyboard shortcuts and category grouping.

use std::ops::{Deref, DerefMut};

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};

use crate::termrock::empty::Readiness;
use crate::termrock::identity::{Id, ItemKey, Revision};
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::picker::PickerState;
use crate::termrock::response::{ActivationOrigin, Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// Metadata for a single command item in a [`CommandPalette`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItem<'a> {
    pub key: ItemKey,
    pub title: &'a str,
    pub category: Option<&'a str>,
    pub shortcut: Option<&'a str>,
    pub disabled: bool,
}

impl<'a> CommandItem<'a> {
    pub fn new(key: impl Into<ItemKey>, title: &'a str) -> Self {
        Self {
            key: key.into(),
            title,
            category: None,
            shortcut: None,
            disabled: false,
        }
    }

    pub fn category(mut self, cat: &'a str) -> Self {
        self.category = Some(cat);
        self
    }

    pub fn shortcut(mut self, sc: &'a str) -> Self {
        self.shortcut = Some(sc);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Durable view state for [`CommandPalette`].
///
/// Wraps and dereferences to [`PickerState`] to share the underlying picker engine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandPaletteState {
    pub picker_state: PickerState,
}

impl CommandPaletteState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_picker_state(picker_state: PickerState) -> Self {
        Self { picker_state }
    }
}

impl Deref for CommandPaletteState {
    type Target = PickerState;

    fn deref(&self) -> &Self::Target {
        &self.picker_state
    }
}

impl DerefMut for CommandPaletteState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.picker_state
    }
}

impl From<PickerState> for CommandPaletteState {
    fn from(picker_state: PickerState) -> Self {
        Self { picker_state }
    }
}

impl From<CommandPaletteState> for PickerState {
    fn from(state: CommandPaletteState) -> Self {
        state.picker_state
    }
}

/// Typed action emitted by [`CommandPalette`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPaletteAction {
    Execute {
        key: ItemKey,
        origin: ActivationOrigin,
    },
    Dismissed,
}

/// Modal command search and launch palette.
#[derive(Clone)]
pub struct CommandPalette<'a> {
    pub id: Id,
    pub commands: &'a [CommandItem<'a>],
    pub revision: Revision,
    pub query: Option<&'a str>,
    pub readiness: Option<Readiness<'a>>,
    pub patch: StylePatch,
    pub max_rows: u16,
    pub width: Option<u16>,
}

impl<'a> CommandPalette<'a> {
    pub fn new(id: Id, commands: &'a [CommandItem<'a>], revision: Revision) -> Self {
        Self {
            id,
            commands,
            revision,
            query: None,
            readiness: None,
            patch: StylePatch::empty(),
            max_rows: 8,
            width: None,
        }
    }

    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
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

    pub fn row_id(&self, key: ItemKey) -> Id {
        self.id.sub("row").sub(&key.to_string())
    }

    pub fn filtered_commands(&self, query: &str) -> Vec<&'a CommandItem<'a>> {
        if query.is_empty() {
            return self.commands.iter().collect();
        }
        let q = query.to_lowercase();
        self.commands
            .iter()
            .filter(|cmd| {
                cmd.title.to_lowercase().contains(&q)
                    || cmd
                        .category
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || cmd
                        .shortcut
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Handles events and returns command actions using [`CommandPaletteState`].
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CommandPaletteState,
    ) -> Response<CommandPaletteAction> {
        self.update_picker(cx, &mut state.picker_state)
    }

    /// Handles events and returns command actions using raw [`PickerState`].
    pub fn update_picker(
        &self,
        cx: &mut Cx<'_>,
        state: &mut PickerState,
    ) -> Response<CommandPaletteAction> {
        if let Some(q) = self.query
            && state.query != q
        {
            state.query = q.to_string();
        }

        let filtered = self.filtered_commands(&state.query);
        let eligible: Vec<ItemKey> = filtered
            .iter()
            .filter(|cmd| !cmd.disabled)
            .map(|cmd| cmd.key)
            .collect();

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
            && let Some(pos) = filtered.iter().position(|cmd| cmd.key == h)
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
                    if !state.query.is_empty() {
                        state.query.clear();
                        let new_filtered = self.filtered_commands(&state.query);
                        let new_eligible: Vec<ItemKey> = new_filtered
                            .iter()
                            .filter(|cmd| !cmd.disabled)
                            .map(|cmd| cmd.key)
                            .collect();
                        state.highlighted = new_eligible.first().copied();
                        state.scroll.total = new_filtered.len();
                        state.scroll.clamp();
                        cx.request_invalidate(Invalidate::Paint);
                        Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                    } else {
                        cx.request_invalidate(Invalidate::Paint);
                        Response::action(self.id.clone(), CommandPaletteAction::Dismissed)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint)
                    }
                }
                KeyCode::Enter => {
                    if is_status_only || filtered.is_empty() {
                        Response::consumed(self.id.clone())
                    } else if let Some(key) = state.highlighted {
                        cx.request_invalidate(Invalidate::Paint);
                        Response::action(
                            self.id.clone(),
                            CommandPaletteAction::Execute {
                                key,
                                origin: ActivationOrigin::Keyboard,
                            },
                        )
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint)
                    } else {
                        Response::consumed(self.id.clone())
                    }
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
                            && let Some(pos) = filtered.iter().position(|cmd| cmd.key == h)
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
                            && let Some(pos) = filtered.iter().position(|cmd| cmd.key == h)
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
                        if let Some(pos) =
                            filtered.iter().position(|cmd| cmd.key == eligible[new_idx])
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
                        if let Some(pos) =
                            filtered.iter().position(|cmd| cmd.key == eligible[new_idx])
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
                        if let Some(pos) = filtered.iter().position(|cmd| cmd.key == last) {
                            state.scroll.ensure_visible(pos);
                        }
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                }
                KeyCode::Backspace => {
                    if !state.query.is_empty() {
                        state.query.pop();
                        let new_filtered = self.filtered_commands(&state.query);
                        let new_eligible: Vec<ItemKey> = new_filtered
                            .iter()
                            .filter(|cmd| !cmd.disabled)
                            .map(|cmd| cmd.key)
                            .collect();
                        state.highlighted = new_eligible.first().copied();
                        state.scroll.total = new_filtered.len();
                        state.scroll.clamp();
                        cx.request_invalidate(Invalidate::Paint);
                        Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
                    } else {
                        cx.request_invalidate(Invalidate::Paint);
                        Response::action(self.id.clone(), CommandPaletteAction::Dismissed)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint)
                    }
                }
                KeyCode::Char(c)
                    if !k.mods.contains(KeyModifiers::CONTROL)
                        && !k.mods.contains(KeyModifiers::ALT) =>
                {
                    state.query.push(c);
                    let new_filtered = self.filtered_commands(&state.query);
                    let new_eligible: Vec<ItemKey> = new_filtered
                        .iter()
                        .filter(|cmd| !cmd.disabled)
                        .map(|cmd| cmd.key)
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
                let new_filtered = self.filtered_commands(&state.query);
                let new_eligible: Vec<ItemKey> = new_filtered
                    .iter()
                    .filter(|cmd| !cmd.disabled)
                    .map(|cmd| cmd.key)
                    .collect();
                state.highlighted = new_eligible.first().copied();
                state.scroll.total = new_filtered.len();
                state.scroll.clamp();
                cx.request_invalidate(Invalidate::Paint);
                Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint)
            }
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::from(m.pos);
                match m.kind {
                    crate::core::event::MouseKind::Down => {
                        for cmd in &filtered {
                            if cx.contains_point(&self.row_id(cmd.key), pos) {
                                if !cmd.disabled && !is_status_only {
                                    state.highlighted = Some(cmd.key);
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        CommandPaletteAction::Execute {
                                            key: cmd.key,
                                            origin: ActivationOrigin::Pointer,
                                        },
                                    )
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                                }
                                return Response::consumed(self.id.clone());
                            }
                        }
                        if let Some(geom) = cx.published_geometry
                            && let Some(modal_rect) = geom.get(&self.id)
                            && !modal_rect.contains(pos)
                        {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                CommandPaletteAction::Dismissed,
                            )
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

    /// Renders the command palette using [`CommandPaletteState`].
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CommandPaletteState) -> Rect {
        self.draw_picker(ui, area, &state.picker_state)
    }

    /// Renders the command palette using raw [`PickerState`].
    pub fn draw_picker(&self, ui: &mut Ui<'_>, area: Rect, state: &PickerState) -> Rect {
        if area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let filtered = self.filtered_commands(&state.query);
        let header_rows = 3u16;
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

        let bg_style = Style::default()
            .bg(theme.tokens.surface_overlay)
            .fg(theme.tokens.text_primary);
        ui.fill_rect(modal_area, bg_style);

        let title_style = Style::default()
            .fg(theme.tokens.accent)
            .add_modifier(Modifier::BOLD);
        ui.set_string(modal_x + 2, modal_y, "Command Palette", title_style);

        let mut curr_y = modal_y + 1;

        // Query field
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

        // Separator
        let sep_style = Style::default().fg(theme.tokens.border_subtle);
        let sep_line: String = "─".repeat(modal_w.saturating_sub(2) as usize);
        ui.set_string(modal_x + 1, curr_y, &sep_line, sep_style);

        curr_y += 1;

        if let Some(Readiness::Loading) = &self.readiness {
            let load_style = Style::default().fg(theme.tokens.text_muted);
            ui.set_string(modal_x + 2, curr_y, "⠋ Loading commands...", load_style);
        } else if let Some(Readiness::Error(msg)) = &self.readiness {
            let err_style = Style::default().fg(theme.tokens.danger);
            ui.set_string(modal_x + 2, curr_y, &format!("! {msg}"), err_style);
        } else if filtered.is_empty() {
            let empty_style = Style::default().fg(theme.tokens.text_muted);
            ui.set_string(modal_x + 2, curr_y, "No matching commands", empty_style);
        } else {
            let visible_range = state.scroll.visible_range();

            for (row_y, idx) in (curr_y..).zip(visible_range) {
                if idx >= filtered.len() || row_y >= modal_y + modal_h.saturating_sub(footer_rows) {
                    break;
                }
                let cmd = filtered[idx];
                let is_highlighted = state.highlighted == Some(cmd.key);
                let row_area = Rect::new(modal_x + 1, row_y, modal_w.saturating_sub(2), 1);
                ui.register_hit(self.row_id(cmd.key), row_area);

                let row_style = if is_highlighted {
                    Style::default()
                        .bg(theme.tokens.highlight)
                        .fg(theme.tokens.accent)
                        .add_modifier(Modifier::BOLD)
                } else if cmd.disabled {
                    Style::default().fg(theme.tokens.disabled)
                } else {
                    Style::default().fg(theme.tokens.text_primary)
                };

                ui.fill_rect(row_area, row_style);

                let marker = if is_highlighted { "›" } else { " " };
                let lead = format!("{marker} ⌘ ");
                ui.set_string(modal_x + 2, row_y, &lead, row_style);

                let lead_w = lead.chars().count() as u16;
                ui.set_string(modal_x + 2 + lead_w, row_y, cmd.title, row_style);

                let mut trail_offset = modal_w.saturating_sub(3);

                if let Some(sc) = cmd.shortcut {
                    let sc_w = sc.chars().count() as u16;
                    let sc_x = modal_x + trail_offset.saturating_sub(sc_w);
                    let sc_style = if is_highlighted {
                        row_style
                    } else {
                        Style::default().fg(theme.tokens.accent)
                    };
                    ui.set_string(sc_x, row_y, sc, sc_style);
                    trail_offset = trail_offset.saturating_sub(sc_w + 1);
                }

                if let Some(cat) = cmd.category {
                    let cat_w = cat.chars().count() as u16;
                    let cat_x = modal_x + trail_offset.saturating_sub(cat_w);
                    let cat_style = if is_highlighted {
                        row_style
                    } else {
                        Style::default().fg(theme.tokens.text_muted)
                    };
                    if cat_x > modal_x + 2 + lead_w + cmd.title.chars().count() as u16 {
                        ui.set_string(cat_x, row_y, cat, cat_style);
                    }
                }
            }
        }

        let footer_y = modal_y + modal_h.saturating_sub(1);
        let footer_style = Style::default().fg(theme.tokens.text_muted);
        let footer_text = "↑↓ Navigate  ↵ Execute  Esc Dismiss";
        ui.set_string(modal_x + 2, footer_y, footer_text, footer_style);

        modal_area
    }

    /// Measures the preferred dimensions under constraints.
    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let w = self.width.unwrap_or(60).min(constraints.max.width);
        let h = (4 + self.max_rows).min(constraints.max.height);
        constraints.clamp(Size::new(w, h))
    }
}
