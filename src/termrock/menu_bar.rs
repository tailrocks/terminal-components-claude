//! Termrock MenuBar horizontal chrome bar, top-level menus, and dropdown coordination.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Modifier, Style};

use crate::termrock::brand::Brand;
use crate::termrock::identity::{ActionKey, Id, ItemKey, Keyed, Revision};
use crate::termrock::layers::DismissReason;
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::menu::{Menu, MenuAction, MenuItem, MenuState};
use crate::termrock::response::{
    ActivationOrigin, Flow, Input, Invalidate, MouseKind, Response, UpdateCause,
};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::text::width;
use crate::termrock::theme::{ColorLevel, StylePatch};

/// A top-level menu entry in a MenuBar.
#[derive(Debug, Clone)]
pub struct TopMenu<'a> {
    pub key: ItemKey,
    pub label: &'a str,
    pub items: &'a [MenuItem<'a>],
    pub disabled: bool,
}

impl<'a> TopMenu<'a> {
    pub fn new(key: ItemKey, label: &'a str, items: &'a [MenuItem<'a>]) -> Self {
        Self {
            key,
            label,
            items,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a> Keyed for TopMenu<'a> {
    fn key(&self) -> ItemKey {
        self.key
    }
}

/// Durable view state for a MenuBar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuBarState {
    pub selected_menu: Option<ItemKey>,
    pub is_open: bool,
    pub menu_state: MenuState,
    pub last_revision: Option<Revision>,
}

impl MenuBarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn selected_menu(&self) -> Option<ItemKey> {
        self.selected_menu
    }

    pub const fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self, key: ItemKey) {
        self.selected_menu = Some(key);
        self.is_open = true;
        self.menu_state = MenuState::new();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.menu_state = MenuState::new();
    }
}

/// A one-row horizontal chrome MenuBar component.
#[derive(Debug, Clone)]
pub struct MenuBar<'a> {
    pub id: Id,
    pub menus: &'a [TopMenu<'a>],
    pub revision: Revision,
    pub leading: Option<Brand<'a>>,
    pub brand_action: Option<ActionKey>,
    pub trailing: Option<&'a str>,
    pub patch: Option<StylePatch>,
}

impl<'a> MenuBar<'a> {
    pub fn new(id: Id, menus: &'a [TopMenu<'a>], revision: Revision) -> Self {
        Self {
            id,
            menus,
            revision,
            leading: None,
            brand_action: None,
            trailing: None,
            patch: None,
        }
    }

    pub fn leading(mut self, brand: Option<Brand<'a>>) -> Self {
        self.leading = brand;
        self
    }

    pub fn brand_action(mut self, action: ActionKey) -> Self {
        self.brand_action = Some(action);
        self
    }

    pub fn trailing(mut self, metadata: &'a str) -> Self {
        self.trailing = Some(metadata);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    fn enabled_menus(&self) -> Vec<&TopMenu<'a>> {
        self.menus.iter().filter(|m| !m.disabled).collect()
    }

    fn find_menu(&self, key: ItemKey) -> Option<&TopMenu<'a>> {
        self.menus.iter().find(|m| m.key == key)
    }

    pub fn measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let brand_w = self
            .leading
            .as_ref()
            .map_or(0, |b| b.measure(cx, constraints).width);
        let menus_w = self
            .menus
            .iter()
            .map(|m| (width(m.label) + 2) as u16)
            .sum::<u16>();
        let meta_w = self.trailing.map_or(0, |t| (width(t) + 2) as u16);
        let total_w = brand_w + menus_w + meta_w;
        constraints.clamp(Size::new(total_w, 1))
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut MenuBarState) -> Response<MenuAction> {
        let enabled = self.enabled_menus();
        if state.selected_menu.is_none()
            || !enabled.iter().any(|m| Some(m.key) == state.selected_menu)
        {
            state.selected_menu = enabled.first().map(|m| m.key);
        }

        // Check if brand is active and clicked
        if let Some(brand) = self.leading.as_ref().filter(|b| b.interactive) {
            let resp = brand.update(cx);
            if let Some(act) = self.brand_action.filter(|_| resp.action.is_some()) {
                return Response::action(
                    self.id.clone(),
                    MenuAction::Invoke {
                        action: act,
                        origin: ActivationOrigin::Pointer,
                    },
                )
                .with_flow(Flow::Consumed)
                .with_invalidate(Invalidate::Paint);
            }
        }

        // If dropdown is open, handle dropdown interactions first
        let open_menu = if state.is_open {
            state
                .selected_menu
                .and_then(|k| self.find_menu(k).map(|m| (k, m)))
        } else {
            None
        };

        if let Some((sel_key, top_menu)) = open_menu {
            let dropdown = Menu::new(self.id.child(sel_key), top_menu.items, self.revision);

            // Check for Left/Right switching top menus before dropdown captures if no submenu is open
            if let (UpdateCause::Input(Input::Key(k), _), true) =
                (cx.cause(), state.menu_state.submenu_path.is_empty())
            {
                match k.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        if !enabled.is_empty() {
                            let curr_idx =
                                enabled.iter().position(|m| m.key == sel_key).unwrap_or(0);
                            let prev_idx = if curr_idx == 0 {
                                enabled.len().saturating_sub(1)
                            } else {
                                curr_idx - 1
                            };
                            state.selected_menu = Some(enabled[prev_idx].key);
                            state.menu_state = MenuState::new();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        // Only switch if current dropdown item is not a submenu that can open
                        let is_on_submenu = state
                            .menu_state
                            .cursor
                            .and_then(|c| top_menu.items.iter().find(|it| it.key() == Some(c)))
                            .is_some_and(|it| matches!(it, MenuItem::Submenu { .. }));

                        if !is_on_submenu && !enabled.is_empty() {
                            let curr_idx =
                                enabled.iter().position(|m| m.key == sel_key).unwrap_or(0);
                            let next_idx = (curr_idx + 1) % enabled.len();
                            state.selected_menu = Some(enabled[next_idx].key);
                            state.menu_state = MenuState::new();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                    }
                    _ => {}
                }
            }

            // Delegate to Menu
            let menu_resp = dropdown.update(cx, &mut state.menu_state);
            if let Some(action) = menu_resp.action {
                match action {
                    MenuAction::Invoke { .. } => {
                        state.close();
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(self.id.clone(), action)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                    }
                    MenuAction::Dismissed { reason } => {
                        state.close();
                        cx.request_invalidate(Invalidate::Paint);
                        if reason == DismissReason::Escape {
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        return Response::action(self.id.clone(), MenuAction::Dismissed { reason })
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                    }
                }
            }
            if menu_resp.flow == Flow::Consumed {
                return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
            }
        }

        // When closed or handling bar-level input
        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::F(10) => {
                    if let Some(key) = state.selected_menu {
                        state.open(key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if !enabled.is_empty() {
                        let curr_idx = state
                            .selected_menu
                            .and_then(|k| enabled.iter().position(|m| m.key == k))
                            .unwrap_or(0);
                        let prev_idx = if curr_idx == 0 {
                            enabled.len().saturating_sub(1)
                        } else {
                            curr_idx - 1
                        };
                        state.selected_menu = Some(enabled[prev_idx].key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if !enabled.is_empty() {
                        let curr_idx = state
                            .selected_menu
                            .and_then(|k| enabled.iter().position(|m| m.key == k))
                            .unwrap_or(0);
                        let next_idx = (curr_idx + 1) % enabled.len();
                        state.selected_menu = Some(enabled[next_idx].key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Down | KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(key) = state.selected_menu {
                        state.open(key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Esc if state.is_open => {
                    state.close();
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone()).with_invalidate(Invalidate::Paint);
                }
                _ => {}
            },
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::new(m.pos.x, m.pos.y);
                match m.kind {
                    MouseKind::Down | MouseKind::Up => {
                        for menu in self.menus {
                            if !menu.disabled && cx.contains_point(&self.id.child(menu.key), pos) {
                                if m.kind == MouseKind::Up {
                                    if state.is_open && state.selected_menu == Some(menu.key) {
                                        state.close();
                                    } else {
                                        state.open(menu.key);
                                    }
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::consumed(self.id.clone())
                                        .with_invalidate(Invalidate::Paint);
                                }
                                return Response::consumed(self.id.clone())
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Response::bubble(self.id.clone())
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &MenuBarState) -> Rect {
        if area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let bar_rect = Rect::new(area.x, area.y, area.width, 1);

        ui.register_focus(self.id.clone(), true);
        ui.register_hit(self.id.clone(), bar_rect);

        // Bar background
        let bg = theme.tokens.surface;
        ui.fill_rect(bar_rect, Style::new().bg(bg));

        let mut curr_x = bar_rect.x;

        // Draw leading brand
        if let Some(brand) = &self.leading {
            let measure_cx = MeasureCx::new(
                Constraints::loose(Size::new(area.width, 1)),
                ui.theme,
                ColorLevel::TrueColor,
            );
            let brand_size =
                brand.measure(&measure_cx, Constraints::loose(Size::new(area.width, 1)));
            let brand_area = Rect::new(curr_x, bar_rect.y, brand_size.width, 1);
            brand.draw(ui, brand_area);
            curr_x += brand_size.width;
        }

        let mut open_menu_rect: Option<Rect> = None;

        // Draw top menu items
        for menu in self.menus {
            if curr_x >= bar_rect.x + bar_rect.width {
                break;
            }

            let label_w = (width(menu.label) + 2) as u16;
            let item_rect = Rect::new(curr_x, bar_rect.y, label_w, 1);
            ui.register_hit(self.id.child(menu.key), item_rect);

            let is_selected = state.selected_menu == Some(menu.key);
            let is_open = state.is_open && is_selected;

            if is_open {
                open_menu_rect = Some(item_rect);
            }

            let (style_bg, style_fg, modifier) = if menu.disabled {
                (bg, theme.tokens.text_muted, Modifier::empty())
            } else if is_open {
                (
                    theme.tokens.popover,
                    theme.tokens.text_primary,
                    Modifier::BOLD,
                )
            } else if is_selected {
                (
                    theme.tokens.highlight,
                    theme.tokens.text_primary,
                    Modifier::BOLD,
                )
            } else {
                (bg, theme.tokens.text_secondary, Modifier::empty())
            };

            let item_style = Style::new()
                .bg(style_bg)
                .fg(style_fg)
                .add_modifier(modifier);
            ui.fill_rect(item_rect, item_style);
            let label_str = format!(" {} ", menu.label);
            ui.set_string(curr_x, bar_rect.y, &label_str, item_style);

            curr_x += label_w;
        }

        // Draw trailing metadata
        if let Some(meta) = self.trailing {
            let meta_w = (width(meta) + 1) as u16;
            let meta_x = (bar_rect.x + bar_rect.width).saturating_sub(meta_w);
            if meta_x > curr_x + 1 {
                let meta_style = Style::new().bg(bg).fg(theme.tokens.text_muted);
                ui.set_string(meta_x, bar_rect.y, meta, meta_style);
            }
        }

        // Draw open dropdown popover
        let open_top = if state.is_open {
            state
                .selected_menu
                .and_then(|k| self.find_menu(k).map(|m| (k, m)))
        } else {
            None
        };

        if let Some((sel_key, top_menu)) = open_top {
            let dropdown = Menu::new(self.id.child(sel_key), top_menu.items, self.revision);
            let measure_cx = MeasureCx::new(
                Constraints::loose(Size::new(40, area.height.saturating_sub(1))),
                ui.theme,
                ColorLevel::TrueColor,
            );
            let dd_size = dropdown.measure(
                &measure_cx,
                Constraints::loose(Size::new(40, area.height.saturating_sub(1))),
            );

            let dd_x = open_menu_rect
                .map_or(area.x, |r| r.x)
                .min((area.x + area.width).saturating_sub(dd_size.width));
            let dd_y = area.y + 1;
            let dd_area = Rect::new(dd_x, dd_y, dd_size.width, dd_size.height);

            dropdown.draw(ui, dd_area, &state.menu_state);
        }

        bar_rect
    }
}
