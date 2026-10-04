//! Termrock Menu engine, item hierarchy, popovers, submenus, and keyboard/pointer navigation.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Modifier, Style};

use crate::termrock::identity::{ActionKey, Id, ItemKey, Revision};
use crate::termrock::layers::DismissReason;
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::response::{
    ActivationOrigin, Flow, Input, Invalidate, MouseKind, Response, UpdateCause,
};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::text::{truncate, width};
use crate::termrock::theme::{ColorLevel, StylePatch};

/// Parts of a Menu for fine-grained style patching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuPart {
    Container,
    Row,
    Marker,
    Label,
    Shortcut,
    Submenu,
    Separator,
    Status,
}

/// A single item in a Menu hierarchy.
#[derive(Debug, Clone)]
pub enum MenuItem<'a> {
    Action {
        key: ItemKey,
        action: ActionKey,
        label: &'a str,
        shortcut: Option<&'a str>,
        disabled: bool,
        danger: bool,
    },
    Separator,
    Submenu {
        key: ItemKey,
        label: &'a str,
        items: &'a [MenuItem<'a>],
        disabled: bool,
    },
}

impl<'a> MenuItem<'a> {
    pub fn action(key: ItemKey, action: ActionKey, label: &'a str) -> Self {
        Self::Action {
            key,
            action,
            label,
            shortcut: None,
            disabled: false,
            danger: false,
        }
    }

    pub const fn separator() -> Self {
        Self::Separator
    }

    pub fn submenu(key: ItemKey, label: &'a str, items: &'a [MenuItem<'a>]) -> Self {
        Self::Submenu {
            key,
            label,
            items,
            disabled: false,
        }
    }

    pub fn shortcut(mut self, sc: &'a str) -> Self {
        if let Self::Action {
            ref mut shortcut, ..
        } = self
        {
            *shortcut = Some(sc);
        }
        self
    }

    pub fn disabled(mut self, dis: bool) -> Self {
        match self {
            Self::Action {
                ref mut disabled, ..
            } => *disabled = dis,
            Self::Submenu {
                ref mut disabled, ..
            } => *disabled = dis,
            Self::Separator => {}
        }
        self
    }

    pub fn danger(mut self, dang: bool) -> Self {
        if let Self::Action { ref mut danger, .. } = self {
            *danger = dang;
        }
        self
    }

    pub const fn key(&self) -> Option<ItemKey> {
        match self {
            Self::Action { key, .. } | Self::Submenu { key, .. } => Some(*key),
            Self::Separator => None,
        }
    }

    pub const fn is_eligible(&self) -> bool {
        match self {
            Self::Action { disabled, .. } => !*disabled,
            Self::Submenu { disabled, .. } => !*disabled,
            Self::Separator => false,
        }
    }

    pub const fn action_key(&self) -> Option<ActionKey> {
        match self {
            Self::Action { action, .. } => Some(*action),
            Self::Submenu { .. } | Self::Separator => None,
        }
    }

    pub const fn is_separator(&self) -> bool {
        matches!(self, Self::Separator)
    }

    pub const fn label(&self) -> Option<&'a str> {
        match self {
            Self::Action { label, .. } | Self::Submenu { label, .. } => Some(*label),
            Self::Separator => None,
        }
    }

    pub const fn shortcut_text(&self) -> Option<&'a str> {
        match self {
            Self::Action { shortcut, .. } => *shortcut,
            Self::Submenu { .. } | Self::Separator => None,
        }
    }
}

/// Action emitted by Menu interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    Invoke {
        action: ActionKey,
        origin: ActivationOrigin,
    },
    Dismissed {
        reason: DismissReason,
    },
}

impl MenuAction {
    pub const fn invoke(action: ActionKey, origin: ActivationOrigin) -> Self {
        Self::Invoke { action, origin }
    }

    pub const fn dismissed(reason: DismissReason) -> Self {
        Self::Dismissed { reason }
    }

    pub const fn action(&self) -> Option<ActionKey> {
        match self {
            Self::Invoke { action, .. } => Some(*action),
            Self::Dismissed { .. } => None,
        }
    }

    pub const fn is_dismissed(&self) -> bool {
        matches!(self, Self::Dismissed { .. })
    }
}

/// Durable view state for a Menu.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuState {
    pub cursor: Option<ItemKey>,
    pub submenu_path: Vec<ItemKey>,
    pub scroll: ScrollState,
    pub last_revision: Option<Revision>,
}

impl MenuState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cursor(mut self, cursor: ItemKey) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub const fn cursor(&self) -> Option<ItemKey> {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: Option<ItemKey>) {
        self.cursor = cursor;
    }

    pub fn submenu_path(&self) -> &[ItemKey] {
        &self.submenu_path
    }

    pub fn open_submenu(&mut self, key: ItemKey) {
        if !self.submenu_path.contains(&key) {
            self.submenu_path.push(key);
        }
    }

    pub fn close_deepest_submenu(&mut self) -> bool {
        self.submenu_path.pop().is_some()
    }
}

/// A popover command-list Menu component.
#[derive(Debug, Clone)]
pub struct Menu<'a> {
    pub id: Id,
    pub items: &'a [MenuItem<'a>],
    pub revision: Revision,
    pub title: Option<&'a str>,
    pub patch: Option<StylePatch>,
    pub part_patches: Vec<(MenuPart, StylePatch)>,
}

impl<'a> Menu<'a> {
    pub fn new(id: Id, items: &'a [MenuItem<'a>], revision: Revision) -> Self {
        Self {
            id,
            items,
            revision,
            title: None,
            patch: None,
            part_patches: Vec::new(),
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub fn patch_part(mut self, part: MenuPart, patch: StylePatch) -> Self {
        self.part_patches.push((part, patch));
        self
    }

    fn eligible_keys(&self) -> Vec<ItemKey> {
        self.items
            .iter()
            .filter(|it| it.is_eligible())
            .filter_map(|it| it.key())
            .collect()
    }

    fn find_item(&self, key: ItemKey) -> Option<&MenuItem<'a>> {
        self.items.iter().find(|it| it.key() == Some(key))
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let title_w = self.title.map_or(0, |t| width(t) + 4);

        let mut max_item_w = 0;
        for item in self.items {
            match item {
                MenuItem::Action {
                    label, shortcut, ..
                } => {
                    let sc_w = shortcut.map_or(0, |s| width(s) + 2);
                    let item_w = width(label) + sc_w + 4; // 2 padding + 2 marker
                    if item_w > max_item_w {
                        max_item_w = item_w;
                    }
                }
                MenuItem::Submenu { label, .. } => {
                    let item_w = width(label) + 6; // marker + label + ' ▶ '
                    if item_w > max_item_w {
                        max_item_w = item_w;
                    }
                }
                MenuItem::Separator => {}
            }
        }

        let width = (title_w.max(max_item_w).max(18) + 2) as u16;
        let mut height = self.items.len() as u16 + 2; // top and bottom border
        if self.title.is_some() {
            height += 2; // title row + separator
        }

        constraints.clamp(Size::new(width, height))
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut MenuState) -> Response<MenuAction> {
        let eligible = self.eligible_keys();

        // Cursor reconciliation
        if state.cursor.is_none() || !eligible.iter().any(|k| Some(*k) == state.cursor) {
            state.cursor = eligible.first().copied();
        }

        // Submenu delegation if open
        let open_sub = state
            .submenu_path
            .last()
            .and_then(|&k| self.find_item(k))
            .and_then(|it| match it {
                MenuItem::Submenu { items, .. } => Some(*items),
                _ => None,
            });

        if let (Some(sub_items), UpdateCause::Input(Input::Key(k), _)) = (open_sub, cx.cause()) {
            match k.code {
                KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                    state.close_deepest_submenu();
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let sub_action = state
                        .cursor
                        .and_then(|c| sub_items.iter().find(|it| it.key() == Some(c)))
                        .and_then(|it| match it {
                            MenuItem::Action {
                                action,
                                disabled: false,
                                ..
                            } => Some(*action),
                            _ => None,
                        });
                    if let Some(action) = sub_action {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(
                            self.id.clone(),
                            MenuAction::Invoke {
                                action,
                                origin: ActivationOrigin::Keyboard,
                            },
                        )
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint);
                    }
                }
                _ => {}
            }
        }

        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if !eligible.is_empty() {
                        let curr_idx = state
                            .cursor
                            .and_then(|c| eligible.iter().position(|k| *k == c))
                            .unwrap_or(0);
                        let prev_idx = if curr_idx == 0 {
                            eligible.len().saturating_sub(1)
                        } else {
                            curr_idx - 1
                        };
                        state.cursor = Some(eligible[prev_idx]);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !eligible.is_empty() {
                        let curr_idx = state
                            .cursor
                            .and_then(|c| eligible.iter().position(|k| *k == c))
                            .unwrap_or(0);
                        let next_idx = (curr_idx + 1) % eligible.len();
                        state.cursor = Some(eligible[next_idx]);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    if let Some(first) = eligible.first() {
                        state.cursor = Some(*first);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::End | KeyCode::Char('G') => {
                    if let Some(last) = eligible.last() {
                        state.cursor = Some(*last);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let sub_key =
                        state
                            .cursor
                            .and_then(|c| self.find_item(c))
                            .and_then(|it| match it {
                                MenuItem::Submenu {
                                    key,
                                    disabled: false,
                                    ..
                                } => Some(*key),
                                _ => None,
                            });
                    if let Some(key) = sub_key {
                        state.open_submenu(key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if !state.submenu_path.is_empty() {
                        state.close_deepest_submenu();
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let current_item = state.cursor.and_then(|c| self.find_item(c));
                    match current_item {
                        Some(MenuItem::Action {
                            action,
                            disabled: false,
                            ..
                        }) => {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                MenuAction::Invoke {
                                    action: *action,
                                    origin: ActivationOrigin::Keyboard,
                                },
                            )
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                        }
                        Some(MenuItem::Submenu {
                            key,
                            disabled: false,
                            ..
                        }) => {
                            state.open_submenu(*key);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    if !state.submenu_path.is_empty() {
                        state.close_deepest_submenu();
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::action(
                        self.id.clone(),
                        MenuAction::Dismissed {
                            reason: DismissReason::Escape,
                        },
                    )
                    .with_flow(Flow::Consumed)
                    .with_invalidate(Invalidate::Paint);
                }
                _ => {}
            },
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::new(m.pos.x, m.pos.y);
                match m.kind {
                    MouseKind::Down | MouseKind::Up => {
                        // Outside check or item hit
                        for item in self.items {
                            let Some(k) = item.key() else { continue };
                            if item.is_eligible() && cx.contains_point(&self.id.child(k), pos) {
                                state.cursor = Some(k);
                                match item {
                                    MenuItem::Action {
                                        action,
                                        disabled: false,
                                        ..
                                    } => {
                                        if m.kind == MouseKind::Up {
                                            cx.request_invalidate(Invalidate::Paint);
                                            return Response::action(
                                                self.id.clone(),
                                                MenuAction::Invoke {
                                                    action: *action,
                                                    origin: ActivationOrigin::Pointer,
                                                },
                                            )
                                            .with_flow(Flow::Consumed)
                                            .with_invalidate(Invalidate::Paint);
                                        }
                                        return Response::consumed(self.id.clone())
                                            .with_invalidate(Invalidate::Paint);
                                    }
                                    MenuItem::Submenu {
                                        key,
                                        disabled: false,
                                        ..
                                    } => {
                                        if m.kind == MouseKind::Up {
                                            state.open_submenu(*key);
                                        }
                                        cx.request_invalidate(Invalidate::Paint);
                                        return Response::consumed(self.id.clone())
                                            .with_invalidate(Invalidate::Paint);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // If mouse up and outside the menu rect, dismiss
                        if m.kind == MouseKind::Up && !cx.contains_point(&self.id, pos) {
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                MenuAction::Dismissed {
                                    reason: DismissReason::OutsidePointer,
                                },
                            )
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Response::bubble(self.id.clone())
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &MenuState) -> Rect {
        if area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let bg = theme.tokens.popover;
        let border_fg = theme.tokens.border_subtle;
        let border_style = Style::new().bg(bg).fg(border_fg);

        ui.register_focus(self.id.clone(), true);
        ui.register_hit(self.id.clone(), area);

        // Fill background
        ui.fill_rect(area, Style::new().bg(bg));

        let right_x = area.x + area.width.saturating_sub(1);
        let bottom_y = area.y + area.height.saturating_sub(1);

        // Borders
        ui.set_string(area.x, area.y, "┌", border_style);
        ui.set_string(right_x, area.y, "┐", border_style);
        ui.set_string(area.x, bottom_y, "└", border_style);
        ui.set_string(right_x, bottom_y, "┘", border_style);

        for x in (area.x + 1)..right_x {
            ui.set_string(x, area.y, "─", border_style);
            ui.set_string(x, bottom_y, "─", border_style);
        }
        for y in (area.y + 1)..bottom_y {
            ui.set_string(area.x, y, "│", border_style);
            ui.set_string(right_x, y, "│", border_style);
        }

        let mut curr_y = area.y + 1;

        // Title row
        if let Some(t) = self.title.filter(|_| curr_y < bottom_y) {
            let max_w = (area.width.saturating_sub(4)) as usize;
            let tr_title = truncate(t, max_w);
            let title_style = Style::new()
                .bg(bg)
                .fg(theme.tokens.text_primary)
                .add_modifier(Modifier::BOLD);
            ui.set_string(area.x + 2, curr_y, &tr_title, title_style);
            curr_y += 1;

            if curr_y < bottom_y {
                ui.set_string(area.x, curr_y, "├", border_style);
                ui.set_string(right_x, curr_y, "┤", border_style);
                for x in (area.x + 1)..right_x {
                    ui.set_string(x, curr_y, "─", border_style);
                }
                curr_y += 1;
            }
        }

        // Draw items
        for item in self.items {
            if curr_y >= bottom_y {
                break;
            }

            let row_rect = Rect::new(area.x + 1, curr_y, area.width.saturating_sub(2), 1);

            match item {
                MenuItem::Separator => {
                    ui.set_string(area.x, curr_y, "├", border_style);
                    ui.set_string(right_x, curr_y, "┤", border_style);
                    for x in (area.x + 1)..right_x {
                        ui.set_string(x, curr_y, "─", border_style);
                    }
                }
                MenuItem::Action {
                    key,
                    label,
                    shortcut,
                    disabled,
                    danger,
                    ..
                } => {
                    ui.register_hit(self.id.child(*key), row_rect);
                    let is_selected = state.cursor == Some(*key);

                    let (row_bg, row_fg, modifier) = if *disabled {
                        (bg, theme.tokens.text_muted, Modifier::empty())
                    } else if is_selected {
                        if *danger {
                            (
                                theme.tokens.highlight_danger,
                                theme.tokens.text_primary,
                                Modifier::BOLD,
                            )
                        } else {
                            (
                                theme.tokens.highlight,
                                theme.tokens.text_primary,
                                Modifier::BOLD,
                            )
                        }
                    } else if *danger {
                        (bg, theme.tokens.danger, Modifier::empty())
                    } else {
                        (bg, theme.tokens.text_primary, Modifier::empty())
                    };

                    let row_style = Style::new().bg(row_bg).fg(row_fg).add_modifier(modifier);
                    ui.fill_rect(row_rect, row_style);

                    // Marker
                    let marker = if is_selected { "› " } else { "  " };
                    ui.set_string(row_rect.x, curr_y, marker, row_style);

                    // Label
                    let label_x = row_rect.x + 2;
                    let avail_w = row_rect.width.saturating_sub(2);
                    let sc_len = shortcut.map_or(0, |s| width(s) + 2) as u16;
                    let label_avail = avail_w.saturating_sub(sc_len) as usize;
                    let tr_label = truncate(label, label_avail);
                    ui.set_string(label_x, curr_y, &tr_label, row_style);

                    // Shortcut
                    if let Some(sc) = shortcut {
                        let sc_w = width(sc) as u16;
                        let sc_x = row_rect.x + row_rect.width.saturating_sub(sc_w + 1);
                        if sc_x > label_x + width(&tr_label) as u16 {
                            let sc_style = Style::new().bg(row_bg).fg(if *disabled {
                                theme.tokens.text_muted
                            } else {
                                theme.tokens.text_secondary
                            });
                            ui.set_string(sc_x, curr_y, sc, sc_style);
                        }
                    }
                }
                MenuItem::Submenu {
                    key,
                    label,
                    disabled,
                    ..
                } => {
                    ui.register_hit(self.id.child(*key), row_rect);
                    let is_selected = state.cursor == Some(*key);
                    let is_open = state.submenu_path.contains(key);

                    let (row_bg, row_fg, modifier) = if *disabled {
                        (bg, theme.tokens.text_muted, Modifier::empty())
                    } else if is_selected || is_open {
                        (
                            theme.tokens.highlight,
                            theme.tokens.text_primary,
                            Modifier::BOLD,
                        )
                    } else {
                        (bg, theme.tokens.text_primary, Modifier::empty())
                    };

                    let row_style = Style::new().bg(row_bg).fg(row_fg).add_modifier(modifier);
                    ui.fill_rect(row_rect, row_style);

                    let marker = if is_selected { "› " } else { "  " };
                    ui.set_string(row_rect.x, curr_y, marker, row_style);

                    let label_x = row_rect.x + 2;
                    let label_avail = row_rect.width.saturating_sub(5) as usize;
                    let tr_label = truncate(label, label_avail);
                    ui.set_string(label_x, curr_y, &tr_label, row_style);

                    // Submenu indicator arrow
                    let arrow_x = row_rect.x + row_rect.width.saturating_sub(2);
                    ui.set_string(arrow_x, curr_y, "▶", row_style);
                }
            }

            curr_y += 1;
        }

        // Draw open child submenus if path is active
        for &sub_key in &state.submenu_path {
            if let Some(MenuItem::Submenu {
                key,
                items: sub_items,
                ..
            }) = self.find_item(sub_key)
            {
                // Find row y of this submenu item
                let sub_idx = self
                    .items
                    .iter()
                    .position(|it| it.key() == Some(sub_key))
                    .unwrap_or(0);
                let title_offset = if self.title.is_some() { 2 } else { 0 };
                let anchor_y = area.y + 1 + title_offset + sub_idx as u16;

                let sub_menu = Menu::new(self.id.child(*key), sub_items, self.revision);
                let measure_cx = MeasureCx::new(
                    Constraints::loose(Size::new(40, 20)),
                    ui.theme,
                    ColorLevel::TrueColor,
                );
                let sub_size = sub_menu.measure(&measure_cx, Constraints::loose(Size::new(40, 20)));

                let sub_x = area.x + area.width.saturating_sub(1);
                let sub_area = Rect::new(sub_x, anchor_y, sub_size.width, sub_size.height);
                sub_menu.draw(ui, sub_area, state);
            }
        }

        area
    }
}
